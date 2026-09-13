# Testes Caixa Preta

Uma ferramenta de linha de comando simples em Rust para execução de testes de caixa-preta (black-box) baseados em arquivos YAML.

## Como funciona

O programa lê a configuração opcional em `./testes-caixa-preta.yaml` e, em seguida, todos os arquivos `.yaml` presentes no diretório `./testes/` (ignorando os arquivos de snapshot). Os arquivos dentro de `./testes/` contêm exclusivamente cenários. O programa executa os comandos definidos utilizando o shell (`sh -c`) e captura a saída padrão (stdout), o erro padrão (stderr) e o código de saída (exit code).

A configuração da raiz pode definir o servidor a ser iniciado, a URL base e o tempo máximo de espera:

```yaml
servidor: "npx -y serve exemplos/"
url_base: "http://localhost:$PORTA"
tempo_espera: 30
```

Na primeira execução, o programa cria automaticamente um arquivo de snapshot (ex: `arquivo-saídas.yaml`) com os resultados obtidos. Nas execuções subsequentes, o programa compara os resultados atuais com os salvos no snapshot para validar o teste.

## Estrutura de Testes

Os testes devem ser criados em arquivos `.yaml` dentro do diretório `./testes/`. O formato segue uma lista de **cenários**, que obrigatoriamente possuem um nome (`cenário`), e podem ser do tipo comandos encadeados (opcionalmente com uma `entrada`) ou testes de navegador (`navegação`):

```yaml
- cenário: "Teste de echo"
  comandos:
    - echo "Olá, Mundo!"

- cenário: "Teste com entrada e encadeamento"
  entrada: |
    banana
    abacate
  comandos:
    - grep b
    - sort

- cenário: "Teste de captura de tela"
  navegação:
    - navegar para: https://example.com
    - capturar tela: example.png
```

- `cenário`: Nome descritivo do cenário de teste, que será exibido no relatório.
- `comandos`: Lista de comandos a serem rodados no shell (a saída padrão de um é passada como entrada padrão para o próximo).
- `entrada` (opcional): O conteúdo a ser enviado para a entrada padrão (stdin) do primeiro comando.
- `modo` (opcional): Define o esquema de cores do navegador para cenários de navegação. Aceita `"claro"` ou `"escuro"`, sendo `"claro"` o padrão.
- `navegação`: Lista de passos para testes no navegador. Atualmente, os passos podem conter:
  - `navegar para`: A URL da página para acessar.
  - `capturar tela`: O nome do arquivo PNG a ser salvo em `testes/telas/`.

## Pré-requisitos

Para desenvolver ou executar este projeto:
- **Dev Container** (recomendado): basta abrir o repositório no VS Code / editor compatível com Dev Containers (requer Docker).
- **Localmente**: [Rust e Cargo](https://rustup.rs/) (edição 2024 / Rust 1.88+), Chromium (ou `chromium-browser`) e Node.js/npm.

## Como Executar

Com o ambiente pronto (no Dev Container ou localmente), você pode rodar os testes executando:

```bash
cargo run
```

O programa exibirá no terminal o progresso de cada arquivo de teste sendo executado e, no fim, um relatório de quantos testes passaram.