let
  pkgs = import <nixpkgs> {};
  testesCaixaPreta = import ./fonte/default.nix { inherit pkgs; };
in
pkgs.writeShellScriptBin "testar" ''
  export PATH="${pkgs.lib.makeBinPath [ pkgs.nodejs ]}:$PATH"
  exec ${testesCaixaPreta}/bin/testes-caixa-preta "$@"
''
