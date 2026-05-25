{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [
    flip-link
    cargo-show-asm
    cargo-bloat
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    targets = [ "thumbv7em-none-eabi" ];
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-src" "llvm-tools" ];
  };
}
