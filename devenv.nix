{
  pkgs,
  lib,
  ...
}:
{
  dotenv.enable = true;
  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  packages = [
    pkgs.cargo-machete
    pkgs.cargo-audit
    pkgs.cargo-edit
    pkgs.dioxus-cli
    pkgs.wasm-bindgen-cli
    pkgs.lld
    pkgs.tailwindcss
    pkgs.sqlx-cli
    pkgs.binaryen
  ];
}
