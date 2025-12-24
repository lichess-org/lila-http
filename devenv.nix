{ pkgs, inputs, lib, ... }:
let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  lila-http = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
    pname = cargoToml.package.name;
    version = cargoToml.package.version or "0.1.0";
    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;
  };

  n2c = inputs.nix2container.packages.${pkgs.system}.nix2container;
  skopeo-nix2container =
    inputs.nix2container.packages.${pkgs.system}.skopeo-nix2container;

  minimalContainer = n2c.buildImage {
    name = "lila-http";
    tag = "latest";
    copyToRoot = [ lila-http ];
    config = {
      entrypoint =
        [ "/bin/lila-http" ]; # Changed from ${lila-http}/bin/lila-http
    };
  };
in {
  languages.rust.enable = true;
  processes.lila-http.exec = "cargo run --release";

  scripts.container-load.exec = ''
    ${skopeo-nix2container}/bin/skopeo --insecure-policy copy nix:${minimalContainer} docker-daemon:lila-http:latest
  '';

  # Usage: container-push destination [skopeo-args...]
  # Example: container-push docker://ghcr.io/org/repo:tag --dest-creds user:pass
  scripts.container-push.exec = ''
    dest="$1"
    shift
    ${skopeo-nix2container}/bin/skopeo --insecure-policy copy "$@" nix:${minimalContainer} "$dest"
  '';
}
