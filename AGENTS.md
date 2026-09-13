# Instruções do Agente

## Configuração de Ambiente
- Toda a configuração de ambiente, dependências e ferramentas é gerenciada pelo Dev Container.
- Execute os comandos de desenvolvimento e teste diretamente no terminal (como `cargo fmt`, `cargo clippy`, `cargo run`).

## Desenvolvimento em Rust
- Ao modificar o código em Rust, sempre verifique se o código está devidamente formatado rodando `cargo fmt` e sem avisos de linting rodando `cargo clippy`.
- Garanta que a compilação passe e que os testes de caixa-preta continuem funcionando executando `cargo run`.

## Testes Caixa-Preta
- Para adicionar novos testes de caixa-preta, crie ou edite arquivos `.yaml` dentro do diretório `./testes/`.
- O formato dos testes é uma lista de objetos onde cada um contém obrigatoriamente um nome descritivo (`cenário`) e, dependendo do tipo de teste, uma lista de `comandos` de shell (opcionalmente com `entrada`) ou uma lista de `navegação` com passos contendo `navegar para` e `capturar tela` (para cenários de navegador).
- Cada comando deve ser seguido, no próprio cenário, por um mapa com as saídas esperadas. Campos omitidos usam saída vazia e código de saída `0`.
