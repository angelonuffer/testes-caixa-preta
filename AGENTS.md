# Instruções do Agente

## Configuração de Ambiente
- Sempre utilize `nix-shell fonte/shell.nix` (ou `nix develop -f fonte/shell.nix`) para executar comandos que necessitem de dependências de desenvolvimento, ferramentas ou um ambiente específico, garantindo que você está trabalhando no ambiente Nix correto para este projeto.
- Certifique-se de executar os comandos do Nix sempre fora do sandbox (`BypassSandbox: true`).

## Desenvolvimento em Rust
- Ao modificar o código em Rust, sempre verifique se o código está devidamente formatado rodando `cargo fmt` e sem avisos de linting rodando `cargo clippy`.
- Garanta que a compilação passe e que os testes de caixa-preta continuem funcionando executando `nix run -f testar.nix` (ou `cargo run` dentro do ambiente `nix-shell`).

## Testes Caixa-Preta
- Para adicionar novos testes de caixa-preta, crie ou edite arquivos dentro do diretório `./testes/` (`.yaml`, `.md` ou `.nix`). Mantenha a pasta `./testes/` exclusiva para cenários de teste e o arquivo de configuração `caixa-preta.yaml`. O executor de testes é o arquivo `testar.nix` na raiz do projeto.
- O formato dos testes é uma lista de objetos onde cada um contém obrigatoriamente um nome descritivo (`cenário`) e, dependendo do tipo de teste, uma lista de `comandos` de shell (opcionalmente com `entrada`) ou uma lista de `navegação` com passos contendo `navegar para` e `capturar tela` (para cenários de navegador).
- O sistema usa snapshots: na primeira execução (quando não há arquivo gerado), ele roda os comandos e cria um arquivo `-saídas.yaml` com as saídas esperadas. Em execuções seguintes, ele verifica a integridade contra esse snapshot.
