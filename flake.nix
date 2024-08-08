{
  description = "Rust-Leptos development environment for working on rss-newspaper generator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
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
        rustEnv = pkgs.rust-bin.nightly."2024-05-09".default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        
        cargoLeptos = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-leptos";
          version = "0.2.17";

          src = pkgs.fetchFromGitHub {
            owner = "leptos-rs";
            repo = "cargo-leptos";
            rev = "v${version}";
            sha256 = "sha256-W08R1ny4LyzWehnsWSMCRjCxmvV0k7ARVbmZ740hg8w=";
          };

          cargoSha256 = "sha256-kuKsBnmH3bPgwuJ1q49iHMNT47Djx9BKxcMBbJ3zqis=";

          RUSTC = "${rustEnv}/bin/rustc";
          CARGO = "${rustEnv}/bin/cargo";

          doCheck = false;

          meta = {
            description = "Cargo extension for Leptos framework";
            homepage = "https://github.com/leptos-rs/cargo-leptos";
            license = pkgs.lib.licenses.mit;
          };

          buildInputs = [
            rustEnv
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
