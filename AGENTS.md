# Instruções do Agente

## Configuração de Ambiente
- Toda a configuração de ambiente, dependências e ferramentas é gerenciada pelo Dev Container.
- Execute os comandos de desenvolvimento e teste diretamente no terminal, como `npm install` e `npm test`.

## Desenvolvimento em Node.js
- Ao modificar o runner, valide a sintaxe e execute a suíte completa com `npm test`.
- Mantenha o uso do Chromium instalado no ambiente para os cenários de navegação.

## Testes Caixa-Preta
- Para adicionar novos testes de caixa-preta, crie ou edite arquivos `.yaml` dentro do diretório `./testes/`.
- O formato dos testes é uma lista de objetos onde cada um contém obrigatoriamente um nome descritivo (`cenário`) e, dependendo do tipo de teste, uma lista de `comandos` de shell (opcionalmente com `entrada`) ou uma lista de `navegação` com passos contendo `navegar para` e `capturar tela` (para cenários de navegador).
- Cada comando deve ser seguido, no próprio cenário, por um mapa com as saídas esperadas. Campos omitidos usam saída vazia e código de saída `0`.
