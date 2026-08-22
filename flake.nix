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

        testeCaixaPretaPkg = pkgs.rustPlatform.buildRustPackage {
          pname = "testes-caixa-preta";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          CARGO_BUILD_TARGET_DIR = "target";
        };
      in
      {
        packages.default = testeCaixaPretaPkg;

        apps.default = flake-utils.lib.mkApp {
          drv = testeCaixaPretaPkg;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            chromium
            (writeShellScriptBin "chromium-browser" ''
              exec chromium "$@"
            '')
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
