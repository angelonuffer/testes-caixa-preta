# Instruções do Agente

## Configuração de Ambiente
- Sempre utilize `nix develop` para executar comandos que necessitem de dependências de desenvolvimento, ferramentas ou um ambiente específico, garantindo que você está trabalhando no ambiente Nix correto para este projeto.

## Desenvolvimento em Rust
- Ao modificar o código em Rust, sempre verifique se o código está devidamente formatado rodando `cargo fmt` e sem avisos de linting rodando `cargo clippy`.
- Garanta que a compilação passe e que os testes de caixa-preta continuem funcionando executando `cargo run`.

## Testes Caixa-Preta
- Para adicionar novos testes de caixa-preta, crie ou edite arquivos `.yaml` dentro do diretório `./testes/`.
- O formato dos testes é uma lista de casos contendo `comando` e `saída_esperada`.
