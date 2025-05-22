{
  description = "Rust-Leptos development environment for working on rss-newspaper generator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rustEnv = pkgs.rust-bin.nightly."2025-04-01".default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        cargoLeptos = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-leptos";
          version = "v0.2.35";

          src = pkgs.fetchFromGitHub {
            owner = "leptos-rs";
            repo = pname;
            rev = version;
            hash = "sha256-CNktytEm6+5QTPAlxNz07+s7gue9dA5zZM82YQOWFSw=";
          };

          cargoSha256 = "sha256-qHaE4ar5AhPCsGT746XBJ4DiEYP0K+qba9FeGbyY7So=";

          # cargoLock = {
          #   lockFile = ./nix/cargo-leptos-${version}.lock;
          #   outputHashes = {
          #     # "leptos_macro-0.2.11" = "sha256-7g3NfKjWl3dZ5hX6sY9cP1wRzV6oUjM8nQlT2vA0a1E=";
          #     # Add other necessary output hashes here if needed
          #   };
          # };

          RUSTC = "${rustEnv}/bin/rustc";
          CARGO = "${rustEnv}/bin/cargo";

          doCheck = false;

          meta = {
            description = "Cargo extension for Leptos framework";
            homepage = "https://github.com/leptos-rs/cargo-leptos";
            license = pkgs.lib.licenses.mit;
          };

          nativeBuildInputs = [
            rustEnv
            pkgs.perl
            pkgs.pkg-config
          ];

          buildInputs = [
            pkgs.openssl
            pkgs.openssl.dev
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.darwin.apple_sdk.frameworks.CoreServices
          ];
        };

      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            rustEnv
            cargoLeptos
            pkgs.openssl
            pkgs.openssl.dev
            pkgs.pkg-config
            pkgs.nodejs
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.darwin.apple_sdk.frameworks.CoreServices
          ];

          nativeBuildInputs = [ pkgs.pkg-config ];

          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [ pkgs.openssl ]};
          '';
        };
      });
}
