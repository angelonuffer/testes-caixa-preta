{ pkgs ? import <nixpkgs> {} }:

let
  chromiumBrowser = pkgs.writeShellScriptBin "chromium-browser" ''
    exec ${pkgs.chromium}/bin/chromium "$@"
  '';
in
pkgs.rustPlatform.buildRustPackage {
  pname = "testes-caixa-preta";
  version = "0.1.0";
  src = pkgs.lib.cleanSource ../.;
  cargoLock = {
    lockFile = ../Cargo.lock;
  };
  nativeBuildInputs = [ pkgs.rustfmt pkgs.makeWrapper ];
  CARGO_BUILD_TARGET_DIR = "target";

  postInstall = ''
    wrapProgram $out/bin/testes-caixa-preta \
      --prefix PATH : ${pkgs.lib.makeBinPath [
        pkgs.chromium
        chromiumBrowser
        pkgs.nix
      ]}
  '';

  meta = {
    description = "Ferramenta para testes de caixa-preta baseados em YAML, Markdown e Nix";
    mainProgram = "testes-caixa-preta";
  };
}
