# Testes Caixa Preta

Uma ferramenta de linha de comando simples em Rust para execução de testes de caixa-preta (*black-box*) baseados em arquivos YAML, Markdown e expressões Nix.

## Como funciona

O programa lê iterativamente todos os arquivos `.yaml`, `.md` e `.nix` presentes no diretório `./testes/` (ignorando os arquivos de snapshot e configurações). Ele executa os comandos definidos utilizando o shell (`sh -c`) ou passos de navegação headless no Chromium e captura as saídas, códigos de saída e capturas de tela.

Na primeira execução de cenários sem validação explícita de snapshot, o programa cria automaticamente um arquivo de snapshot (ex: `arquivo-saídas.yaml`) com os resultados obtidos. Nas execuções subsequentes, o programa compara os resultados atuais com os salvos no snapshot para validar o teste.

## Estrutura de Testes

Os testes devem ser criados dentro do diretório `./testes/`. O formato segue uma lista de **cenários**, que obrigatoriamente possuem um nome (`cenário`), e podem ser comandos de shell encadeados ou testes de navegador (`navegação`):

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

- `cenário`: Nome descritivo do cenário de teste exibido no relatório.
- `comandos`: Lista de comandos a serem rodados no shell (a saída padrão de um é enviada como entrada padrão para o próximo).
- `entrada` (opcional): O conteúdo a ser enviado para a entrada padrão (stdin) do primeiro comando.
- `modo` (opcional): Define o esquema de cores do navegador (`"claro"` ou `"escuro"`, sendo `"claro"` o padrão).
- `navegação`: Lista de passos para testes no navegador:
  - `navegar para`: URL da página a ser acessada.
  - `capturar tela`: Nome do arquivo PNG salvo em `testes/telas/`.
  - `esperar aparecer` / `esperar sumir`: Aguarda determinado texto no DOM.
  - `enviar formulário`: Preenche campos de formulário antes de submeter.

---

## Como Executar Neste Repositório

### Executando os Testes

Para executar a suíte de testes de caixa-preta deste projeto, basta rodar o comando na raiz:

```sh
nix run -f testar.nix
```

O Nix compilará a ferramenta (com Chromium e dependências embutidos) e executará todos os testes presentes em `./testes/`.

### Ambiente de Desenvolvimento

Para entrar no ambiente de desenvolvimento com todas as dependências de compilação configuradas (`cargo`, `rustc`, `rustfmt`, `clippy`, `rust-analyzer`, `chromium`, `nodejs`):

```sh
nix-shell fonte/shell.nix
```

Ou usando o CLI experimental do Nix:

```sh
nix develop -f fonte/shell.nix
```

Dentro do shell, você pode utilizar os comandos tradicionais do ecossistema Rust:

```sh
cargo check
cargo run
cargo fmt
cargo clippy
```

---

## Como Importar e Usar em Outros Repositórios

Você pode utilizar o `testes-caixa-preta` em qualquer outro projeto sem precisar clonar ou instalar a ferramenta manualmente.

### 1. Organize seus cenários de teste

No repositório do seu projeto, crie o diretório `./testes/` e coloque seus arquivos de cenários (`.yaml`, `.md` ou `.nix`), mantendo a pasta `testes/` exclusiva para eles.

### 2. Crie o arquivo `testar.nix` na raiz do seu projeto

Crie o arquivo `testar.nix` na raiz do repositório consumidor:

#### Exemplo Básico (sem dependências extras no PATH)

```nix
let
  pkgs = import <nixpkgs> {};
  testesCaixaPretaRepo = builtins.fetchGit {
    url = "https://github.com/angelonuffer/testes-caixa-preta";
    # ref = "main"; # opcional, padrão é a branch padrão
    # rev = "...";  # opcional, para fixar um commit específico
  };
  # pkgs é opcional: se omitido ({}), utiliza a versão fixada de nixpkgs do repositório
  testesCaixaPreta = import "${testesCaixaPretaRepo}/fonte/default.nix" { inherit pkgs; };
in
testesCaixaPreta
```

#### Exemplo com Ferramentas Adicionais no PATH

Se os testes ou o servidor do seu projeto precisarem de ferramentas específicas no `PATH` durante a execução (como `nodejs`, `python`, etc.):

```nix
let
  pkgs = import <nixpkgs> {};
  testesCaixaPretaRepo = builtins.fetchGit {
    url = "https://github.com/angelonuffer/testes-caixa-preta";
  };
  testesCaixaPreta = import "${testesCaixaPretaRepo}/fonte/default.nix" { inherit pkgs; };
in
pkgs.writeShellScriptBin "testar" ''
  export PATH="${pkgs.lib.makeBinPath [ pkgs.nodejs ]}:$PATH"
  exec ${testesCaixaPreta}/bin/testes-caixa-preta "$@"
''
```

### 3. Execute os testes no seu projeto

No repositório do seu projeto, basta executar:

```sh
nix run -f testar.nix
```