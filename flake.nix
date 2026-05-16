{
  description = "A language server for just";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        package = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = package.name;
          version = package.version;

          src = ./.;

          auditable = false;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          doCheck = false;

          meta = {
            description = package.description;
            homepage = package.homepage;
            changelog = "${package.repository}/blob/master/CHANGELOG.md";
            license = pkgs.lib.licenses.cc0;
            mainProgram = package.name;
          };
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
          ];
        };
      }
    );
}
