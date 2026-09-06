{
  config ? {},
  overlays ? [],
  ...
}@args:

let
  lock = {
    rev = "0e251e24a4f24e036a084b6b4b2d2491af4167f4";
    sha256 = "sha256-yNJd40f11EzXBjSByCB7IPpeFFAdeoSKKM67dGkfFoU=";
  };
  tarball = fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${lock.rev}.tar.gz";
    sha256 = lock.sha256;
  };
in
import tarball args
