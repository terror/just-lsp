{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = with pkgs;
          let
            darwinInclude = lib.optionalString stdenv.isDarwin ''- "-I${darwin.Libsystem}/include/"'';
          in
          mkShell {
            packages = [ tree-sitter nodejs-slim_22 graphviz ];
            shellHook = ''
              cat <<EOF > .clangd
              CompileFlags:
                Add:
                - "-I${clang}/resource-root/include/"
                ${darwinInclude}
              EOF
            '';
          };
      });
}
