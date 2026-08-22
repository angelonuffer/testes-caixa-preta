# Testes Caixa Preta

Uma ferramenta de linha de comando simples em Rust para execução de testes de caixa-preta (black-box) baseados em arquivos YAML.

## Como funciona

O programa lê iterativamente todos os arquivos `.yaml` presentes no diretório `./testes/`. Ele executa os comandos definidos utilizando o shell (`sh -c`) e compara a saída (stdout) resultante com o valor esperado estipulado no arquivo.

## Estrutura de Testes

Os testes devem ser criados em arquivos `.yaml` dentro do diretório `./testes/`. O formato segue uma lista de casos, conforme o exemplo abaixo:

```yaml
- comando: echo "Olá, Mundo!"
  saída_esperada: Olá, Mundo!
```

- `comando`: O comando a ser rodado no shell.
- `saída_esperada`: O valor de saída padrão (stdout) que deve ser verificado para considerar o teste como aprovado.

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