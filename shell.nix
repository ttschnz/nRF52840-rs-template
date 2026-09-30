{ pkgs ? import <nixpkgs> {} }:

let
  rust-overlay = import (
    builtins.fetchTarball {
      url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
    }
  );

  pkgs' = import <nixpkgs> {
    overlays = [ rust-overlay ];
  };

  rust = pkgs'.rust-bin.stable.latest.default.override {
    targets = [
      "thumbv7em-none-eabihf"
    ];
  };

  uf2conv = pkgs'.rustPlatform.buildRustPackage rec {
    pname = "uf2conv";
    version = "git";

    src = pkgs'.fetchFromGitHub {
      owner = "ttschnz";
      repo = "uf2conv-rs";
      rev = "b2ea5c348b0c37ea8ce92a3573d796eff89a93ab";
      hash = "sha256-B3GQawutdrMeXlqOiuAK2hlINnIId8B5K3zdlaJyBuw=";
    };

    buildAndTestSubdir = "bin";
    cargoHash = "sha256-WJzN1jBGkZ21x9ueXDLoDb9XCe7NCP/jEXLAvHS/4xQ==";
  };
in
pkgs'.mkShell {
  packages = with pkgs'; [
    rust
    probe-rs-tools
    llvm
    binutils
    cargo-binutils
    uf2conv
    rust-script
  ];
}