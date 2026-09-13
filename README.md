# Testes Caixa Preta

Uma ferramenta de linha de comando simples em Rust para execução de testes de caixa-preta (black-box) baseados em arquivos YAML.

## Como funciona

O programa lê a configuração opcional em `./testes-caixa-preta.yaml` e, em seguida, todos os arquivos `.yaml` presentes no diretório `./testes/` (ignorando arquivos terminados em `-saídas.yaml`). Os arquivos dentro de `./testes/` contêm exclusivamente cenários. O programa executa os comandos definidos utilizando o shell (`sh -c`) e compara a saída padrão (stdout), o erro padrão (stderr) e o código de saída (exit code) com as expectativas declaradas no próprio cenário.

A configuração da raiz pode definir o servidor a ser iniciado, a URL base e o tempo máximo de espera:

```yaml
servidor: "miniserve -p 0 exemplos/"
url_base: "http://localhost:$PORTA"
tempo_espera: 30
```

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
    - saída padrão: |
      banana
      abacate
    - sort
    - saída padrão: |
      abacate
      banana

- cenário: "Teste de captura de tela"
  navegação:
    - simular data: 2024-05-06T07:08:09.000Z
    - navegar para: https://example.com
    - capturar tela: example.png
```

- `cenário`: Nome descritivo do cenário de teste, que será exibido no relatório.
- `comandos`: Lista alternada de comandos e suas expectativas. Cada comando deve ser seguido por um mapa com `saída padrão`, `erro padrão` e `código saída`; a saída padrão de um comando é passada como entrada padrão para o próximo.
- `entrada` (opcional): O conteúdo a ser enviado para a entrada padrão (stdin) do primeiro comando.
- `saída padrão` (opcional): Saída padrão esperada do comando. O padrão é vazio.
- `erro padrão` (opcional): Erro padrão esperado do comando. O padrão é vazio.
- `código saída` (opcional): Código de saída esperado do comando. O padrão é `0`.
- `modo` (opcional): Define o esquema de cores do navegador para cenários de navegação. Aceita `"claro"` ou `"escuro"`, sendo `"claro"` o padrão.
- `navegação`: Lista de passos para testes no navegador. Atualmente, os passos podem conter:
  - `simular data`: Define a data usada por `new Date()` e `Date.now()` a partir desse passo.
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