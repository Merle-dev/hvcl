{
  description = "Rust Calculator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        # This builds your executable
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "hvcl";
          version = "0.1.0";
          src = ./.; # Points to your source code

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          # This hash ensures the dependencies haven't changed.
          # Set this to lib.fakeSha256 first, run nix build,
          # then replace it with the actual hash Nix provides.
          cargoSha256 = "sha256-ZSQD8ou9HbKow5gRcYrvWZ2pdsrrJnDopNQU7I7JAMY";
          cargoHash = "sha256-ZSQD8ou9HbKow5gRcYrvWZ2pdsrrJnDopNQU7I7JAMY";
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            cargo
            rustc
            gtk4
            gtk4-layer-shell
            glib
            gdk-pixbuf
            pango
            cairo
            graphene
            libadwaita
          ];
        };

        # This lets you run 'nix develop' to get a dev environment
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            pkg-config
            gtk4
            gtk4-layer-shell
          ];
        };
      });
}
