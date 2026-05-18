{
  pkgs ? import <nixpkgs> { },
}:
let
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
in
pkgs.callPackage (
  {
    stdenv,
    mkShell,
    rustup,
    rustPlatform,
    inetutils,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = [
      rustup
      rustPlatform.bindgenHook
      inetutils
    ];
    # libraries here
    buildInputs =
      [
      ];
    RUSTC_VERSION = overrides.toolchain.channel;
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    shellHook = ''
      export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
      export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
    '';
  }
) { }
