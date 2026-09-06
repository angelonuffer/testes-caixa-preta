{ pkgs ? import ./nixpkgs.nix {} }:

pkgs.mkShell {
  inputsFrom = [ (import ./default.nix { inherit pkgs; }) ];
  buildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    chromium
    nodejs
  ];

  RUST_BACKTRACE = 1;

  shellHook = ''
    export PS1="\n\[\033[1;32m\][nix-shell:\w]\$\[\033[0m\] "
  '';
}
