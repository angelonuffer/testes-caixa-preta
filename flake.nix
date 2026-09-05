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

        chromiumBrowser = pkgs.writeShellScriptBin "chromium-browser" ''
          exec ${pkgs.chromium}/bin/chromium "$@"
        '';

        testeCaixaPretaPkg = pkgs.rustPlatform.buildRustPackage {
          pname = "testes-caixa-preta";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [ pkgs.rustfmt pkgs.makeWrapper ];
          CARGO_BUILD_TARGET_DIR = "target";

          postInstall = ''
            wrapProgram $out/bin/testes-caixa-preta \
              --prefix PATH : ${pkgs.lib.makeBinPath [
                pkgs.chromium
                chromiumBrowser
              ]}
          '';
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
            chromiumBrowser
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
