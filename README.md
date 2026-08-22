# Testes Caixa Preta

Uma ferramenta de linha de comando simples em Rust para execução de testes de caixa-preta (black-box) baseados em arquivos YAML.

## Como funciona

O programa lê iterativamente todos os arquivos `.yaml` presentes no diretório `./testes/` (ignorando os arquivos de snapshot). Ele executa os comandos definidos utilizando o shell (`sh -c`) e captura a saída padrão (stdout), o erro padrão (stderr) e o código de saída (exit code).

Na primeira execução, o programa cria automaticamente um arquivo de snapshot (ex: `arquivo-saídas.yaml`) com os resultados obtidos. Nas execuções subsequentes, o programa compara os resultados atuais com os salvos no snapshot para validar o teste.

## Estrutura de Testes

Os testes devem ser criados em arquivos `.yaml` dentro do diretório `./testes/`. O formato segue uma lista de **cenários**, onde você pode especificar um comando único ou múltiplos comandos encadeados, e opcionalmente uma entrada:

```yaml
- comando: echo "Olá, Mundo!"

- entrada: |
    banana
    abacate
  comandos:
    - grep b
    - sort
```

- `comando`: O comando a ser rodado no shell (para cenários de um único passo).
- `comandos`: Lista de comandos a serem rodados no shell (para cenários de múltiplos passos onde a saída de um é a entrada do próximo).
- `entrada` (opcional): O conteúdo a ser enviado para a entrada padrão (stdin) do cenário.

## Execução Externa

Para rodar os testes externamente, sem precisar clonar o repositório, você pode utilizar o seguinte comando:

```sh
nix run github:angelonuffer/testes-caixa-preta
```

## Pré-requisitos

Para desenvolver ou executar este projeto localmente a partir do código-fonte, você precisará ter instalado:
- [Nix](https://nixos.org/download.html) com suporte a *Flakes* ativado (fortemente recomendado) ou
- [Rust e Cargo](https://rustup.rs/).

## Como Executar

### Utilizando Nix (Recomendado)

O projeto contém um `flake.nix` já configurado com um ambiente padronizado. Para acessá-lo, use:

```bash
nix develop
```

Este comando ativará um shell com todas as dependências (`cargo`, `rustc`, `clippy`, etc) instaladas e configuradas.

### Compilando e Rodando

Com o ambiente pronto, você pode rodar os testes executando:

```bash
cargo run
```

O programa exibirá no terminal o progresso de cada arquivo `.yaml` sendo testado e, no fim, um relatório de quantos testes passaram.