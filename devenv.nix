{
  pkgs,
  lib,
  ...
}:
{
  dotenv.enable = true;
  languages = {
    javascript = {
      enable = true;
      package = pkgs.nodejs_24;
      npm.enable = true;
    };
    rust = {
      enable = true;
      channel = "nightly";
      version = "2026-08-05";
      targets = [ "wasm32-unknown-unknown" ];
    };
  };

  packages = [
    pkgs.cargo-machete
    pkgs.cargo-audit
    pkgs.cargo-edit
    pkgs.dioxus-cli
    pkgs.wasm-bindgen-cli
    pkgs.lld
    pkgs.sqlx-cli
    pkgs.binaryen
  ];
}
