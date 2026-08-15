{
  description = "A standard Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];

          # Environment variables
          RUST_BACKTRACE = 1;

          shellHook = ''
            export PS1="\n\[\033[1;32m\][nix-shell:\w]\$\[\033[0m\] "
          '';
        };
      }
    );
}
