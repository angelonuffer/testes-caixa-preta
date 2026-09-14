import crypto from 'node:crypto';
import fs from 'node:fs';
import net from 'node:net';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';
import YAML from 'yaml';

const raiz = path.dirname(fileURLToPath(import.meta.url));
const testesDir = path.join(raiz, 'testes');
const configPath = path.join(raiz, 'testes-caixa-preta.yaml');
const reset = '\x1b[0m';
const red = '\x1b[1;31m';
const green = '\x1b[1;32m';

function substituirPorta(texto, porta) {
  return texto.replaceAll('${PORTA}', String(porta)).replaceAll('$PORTA', String(porta));
}

function lerYaml(caminho) {
  return YAML.parse(fs.readFileSync(caminho, 'utf8'));
}

function formatarSaida(valor) {
  const texto = String(valor ?? '');
  return texto ? texto.split('\n').map((linha) => `        | ${linha}`).join('\n') : '        (vazio)';
}

function executarComandos(cenario) {
  process.stdout.write(`  \x1b[1m${cenario['cenário']}\x1b[0m ... `);
  let entrada = cenario.entrada ?? '';
  const obtidos = [];
  const esperados = [];
  let falhou = false;

  for (const passo of cenario.comandos ?? []) {
    if (typeof passo !== 'string') {
      esperados.push({
        'saída padrão': passo['saída padrão'] ?? passo.saída_padrão ?? '',
        'erro padrão': passo['erro padrão'] ?? passo.erro_padrão ?? '',
        'código saída': passo['código saída'] ?? passo.código_saída ?? 0,
      });
      continue;
    }

    const resultado = spawnSync(passo, {
      shell: '/bin/sh',
      input: entrada,
      encoding: 'utf8',
    });
    if (resultado.error) {
      console.log(`${red}❌ FALHOU${reset} (erro ao iniciar processo: ${resultado.error.message})`);
      falhou = true;
      break;
    }
    const obtido = {
      'saída padrão': (resultado.stdout ?? '').trim(),
      'erro padrão': (resultado.stderr ?? '').trim(),
      'código saída': resultado.status ?? -1,
    };
    entrada = obtido['saída padrão'];
    obtidos.push(obtido);
  }

  if (falhou) return false;
  if (esperados.length !== obtidos.length) {
    console.log(`${red}❌ FALHOU${reset} (cada comando deve ser seguido por sua saída esperada)`);
    return false;
  }

  for (let indice = 0; indice < esperados.length; indice += 1) {
    const esperado = esperados[indice];
    const obtido = obtidos[indice];
    const campos = ['saída padrão', 'erro padrão', 'código saída'];
    const diferente = campos.some((campo) => campo === 'código saída'
      ? esperado[campo] !== obtido[campo]
      : String(esperado[campo]).trim() !== obtido[campo]);
    if (diferente) {
      if (!falhou) console.log(`${red}❌ FALHOU${reset}`);
      falhou = true;
      console.log(`    Passo do cenário: ${indice + 1}`);
      for (const campo of campos) {
        const esperadoCampo = campo === 'código saída' ? esperado[campo] : String(esperado[campo]).trim();
        if (esperadoCampo !== obtido[campo]) {
          console.log(`      ${campo} esperado: ${formatarSaida(esperadoCampo)}`);
          console.log(`      ${campo} obtido:   ${formatarSaida(obtido[campo])}`);
        }
      }
    }
  }
  if (falhou) return false;
  console.log(`${green}✅ PASSOU${reset}`);
  return true;
}

function aguardar(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function portasEmEscuta() {
  const resultado = spawnSync('ss', ['-ltnH'], { encoding: 'utf8' });
  if (resultado.status !== 0) return new Set();
  return new Set([...resultado.stdout.matchAll(/(?:\[::\]|0\.0\.0\.0|127\.0\.0\.1|\*):([0-9]+)/g)].map((match) => Number(match[1])));
}

async function descobrirPorta(portasAntes, limiteSegundos) {
  const inicio = Date.now();
  while (Date.now() - inicio < limiteSegundos * 1000) {
    const nova = [...portasEmEscuta()].find((porta) => !portasAntes.has(porta));
    if (nova) return nova;
    await aguardar(100);
  }
  return undefined;
}

function abrirPorta(porta) {
  return new Promise((resolve) => {
    const socket = net.createConnection({ host: '127.0.0.1', port: porta });
    socket.once('connect', () => { socket.destroy(); resolve(true); });
    socket.once('error', () => { socket.destroy(); resolve(false); });
  });
}

async function esperarServidor(url, limiteSegundos, processo) {
  const parsed = new URL(url);
  const porta = Number(parsed.port || (parsed.protocol === 'https:' ? 443 : 80));
  const inicio = Date.now();
  while (Date.now() - inicio < limiteSegundos * 1000) {
    if (processo.exitCode !== null) return false;
    if (await abrirPorta(porta)) return true;
    await aguardar(100);
  }
  return false;
}

async function iniciarServidor(config) {
  if (!config?.servidor) return { config, processo: null };
  let porta;
  let comando = config.servidor;
  if (comando.includes('$PORTA')) {
    const reserva = net.createServer();
    await new Promise((resolve) => reserva.listen(0, '127.0.0.1', resolve));
    porta = reserva.address().port;
    await new Promise((resolve) => reserva.close(resolve));
    comando = substituirPorta(comando, porta);
  }
  console.log(`\x1b[1;36m🚀 Iniciando servidor: ${comando}${reset}`);
  const portasAntes = portasEmEscuta();
  fs.writeFileSync(path.join(testesDir, 'servidor.log'), '');
  const log = fs.openSync(path.join(testesDir, 'servidor.log'), 'a');
  const processo = spawn('/bin/sh', ['-c', `exec ${comando}`], {
    cwd: testesDir,
    detached: true,
    stdio: ['ignore', log, log],
  });
  processo.exitCode = null;
  const limite = config.tempo_espera ?? config.tempo_espera_servidor ?? 30;
  let urlBase = config.url_base;
  if (!porta && urlBase?.includes('$PORTA')) {
    porta = await descobrirPorta(portasAntes, limite);
  }
  if (porta) {
    process.env.PORTA = String(porta);
    urlBase = urlBase ? substituirPorta(urlBase, porta) : undefined;
    console.log(`\x1b[1;32m🔌 Servidor escutando na porta: ${porta}${reset}`);
  }
  if (urlBase) {
    const pronto = await esperarServidor(urlBase, limite, processo);
    if (!pronto) console.error(`\x1b[1;33m⚠️ Aviso: Servidor não parece estar pronto em ${urlBase} após ${limite} segundos.${reset}`);
  } else {
    await aguardar(config.tempo_espera ? config.tempo_espera * 1000 : 1500);
  }
  return { config: { ...config, url_base: urlBase }, processo };
}

async function testarNavegador(cenario, config) {
  process.stdout.write(`  \x1b[1m${cenario['cenário']}\x1b[0m ... `);
  const profileDir = path.join(testesDir, 'chrome-profile');
  fs.rmSync(profileDir, { recursive: true, force: true });
  const telasDir = path.join(testesDir, 'telas');
  fs.mkdirSync(telasDir, { recursive: true });
  let context;
  try {
    context = await chromium.launchPersistentContext(profileDir, {
      executablePath: '/usr/bin/chromium-browser',
      headless: true,
      args: ['--no-sandbox', '--disable-gpu', '--allow-file-access-from-files', '--disable-web-security', ...(cenario.modo === 'escuro' ? ['--force-dark-mode'] : [])],
      colorScheme: cenario.modo === 'escuro' ? 'dark' : 'light',
      viewport: { width: 780, height: 437 },
    });
    const page = await context.newPage();
    let dataSimulada;
    for (const passo of cenario.navegação ?? []) {
      if (passo['simular data']) {
        dataSimulada = passo['simular data'];
        await context.addInitScript({ content: scriptData(dataSimulada) });
      }
      if (passo['navegar para']) {
        const endereço = substituirPorta(passo['navegar para'], process.env.PORTA ?? '');
        const url = config?.url_base
          ? `${config.url_base.replace(/\/$/, '')}/${endereço.replace(/^\//, '')}`
          : `file://${path.join(raiz, endereço)}`;
        await page.goto(url, { waitUntil: 'load' });
      }
      if (passo['enviar formulário']) {
        for (const [id, valor] of Object.entries(passo['enviar formulário'])) {
          await page.locator(`#${id}`).fill(valor);
          await page.locator(`#${id}`).evaluate((element) => {
            element.focus();
            element.setSelectionRange(element.value.length, element.value.length);
          });
        }
        await page.evaluate(() => document.querySelector('button[type="submit"]')?.click());
      }
      if (passo['esperar aparecer']) await page.getByText(passo['esperar aparecer'], { exact: false }).waitFor({ state: 'visible', timeout: 5000 });
      if (passo['esperar sumir']) await page.getByText(passo['esperar sumir'], { exact: false }).waitFor({ state: 'hidden', timeout: 5000 });
      if (passo['clicar em'] ?? passo.clicar) {
        const alvo = passo['clicar em'] ?? passo.clicar;
        await page.locator(alvo).or(page.getByText(alvo, { exact: true })).first().click({ timeout: 5000 });
      }
      if (passo['capturar tela']) {
        const dados = await page.screenshot({
          path: path.join(telasDir, passo['capturar tela']),
          caret: 'initial',
        });
        const hash = crypto.createHash('sha256').update(dados).digest('hex');
        if (!passo['hash esperado'] || passo['hash esperado'] !== hash) {
          console.log(`${red}❌ FALHOU${reset}`);
          console.log(`    tela: ${passo['capturar tela']}`);
          if (passo['hash esperado']) console.log(`      hash esperado: ${passo['hash esperado']}\n      hash obtido:   ${hash}`);
          else console.log(`      (a captura não especifica 'hash esperado')`);
          return false;
        }
      }
    }
    console.log(`${green}✅ PASSOU${reset}`);
    return true;
  } catch (erro) {
    console.log(`${red}❌ FALHOU${reset} (${erro.message})`);
    return false;
  } finally {
    if (context) await context.close();
  }
}

function scriptData(data) {
  return `(() => { const NativeDate = globalThis.__testesCaixaPretaNativeDate || Date; globalThis.__testesCaixaPretaNativeDate = NativeDate; const fixed = new NativeDate(${JSON.stringify(data)}).getTime(); class MockDate extends NativeDate { constructor(...args) { super(...(args.length ? args : [fixed])); } static now() { return fixed; } } Object.setPrototypeOf(MockDate, NativeDate); Object.defineProperty(globalThis, 'Date', { configurable: true, writable: true, value: MockDate }); })();`;
}

async function principal() {
  if (!fs.existsSync(testesDir)) {
    console.error(`${red}❌ Diretório ./testes não encontrado.${reset}`);
    process.exitCode = 1;
    return;
  }
  let config;
  if (fs.existsSync(configPath)) {
    try { config = lerYaml(configPath); } catch { config = undefined; }
  }
  const servidor = await iniciarServidor(config);
  let total = 0;
  let passou = 0;
  let errosParse = 0;
  const arquivos = fs.readdirSync(testesDir).filter((nome) => nome.endsWith('.yaml') && !nome.endsWith('-saídas.yaml')).sort();
  try {
    for (const nome of arquivos) {
      const caminho = path.join(testesDir, nome);
      let casos;
      try { casos = lerYaml(caminho); } catch (erro) {
        console.error(`${red}❌ Erro ao fazer parse do arquivo ${path.join('./testes', nome)}: ${erro.message}${reset}`);
        errosParse += 1;
        continue;
      }
      console.log(`\x1b[1;34m📄 ${path.join('./testes', nome)}${reset}`);
      for (const cenario of casos) {
        total += 1;
        passou += (cenario.comandos ? executarComandos(cenario) : await testarNavegador(cenario, servidor.config)) ? 1 : 0;
      }
    }
  } finally {
    if (servidor.processo) {
      try { process.kill(-servidor.processo.pid, 'SIGTERM'); } catch {}
      await aguardar(300);
      try { process.kill(-servidor.processo.pid, 'SIGKILL'); } catch {}
    }
  }
  if (errosParse) {
    console.log(`\n${red}❌ ${errosParse} ${errosParse === 1 ? 'erro' : 'erros'}${reset}`);
    process.exitCode = 1;
  } else {
    const cor = passou === total ? green : red;
    console.log(`\n${cor}${passou === total ? '✅' : '❌'} ${passou}/${total} testes passaram.${reset}`);
    process.exitCode = passou === total ? 0 : 1;
  }
}

principal();