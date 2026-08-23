# Instruções do Agente

## Configuração de Ambiente
- Sempre utilize `nix develop` para executar comandos que necessitem de dependências de desenvolvimento, ferramentas ou um ambiente específico, garantindo que você está trabalhando no ambiente Nix correto para este projeto.

## Desenvolvimento em Rust
- Ao modificar o código em Rust, sempre verifique se o código está devidamente formatado rodando `cargo fmt` e sem avisos de linting rodando `cargo clippy`.
- Garanta que a compilação passe e que os testes de caixa-preta continuem funcionando executando `cargo run`.

## Testes Caixa-Preta
- Para adicionar novos testes de caixa-preta, crie ou edite arquivos `.yaml` dentro do diretório `./testes/`.
- O formato dos testes é uma lista de objetos onde cada um contém obrigatoriamente um nome descritivo (`cenário`) e, dependendo do tipo de teste, uma lista de `comandos` de shell (opcionalmente com `entrada`) ou uma lista de `navegação` com passos contendo `endereço` (para cenários de navegador).
- O sistema usa snapshots: na primeira execução (quando não há arquivo gerado), ele roda os comandos e cria um arquivo `-saídas.yaml` com as saídas esperadas. Em execuções seguintes, ele verifica a integridade contra esse snapshot.
