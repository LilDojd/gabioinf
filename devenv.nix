{
  pkgs,
  inputs,
  ...
}:
let
  wasmTooling = inputs.wasm_bindgen_cli.legacyPackages.${pkgs.stdenv.hostPlatform.system};
in
{
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
    # Match the wasm-bindgen crate version locked by Dioxus.
    wasmTooling.wasm-bindgen-cli
    pkgs.flyctl
    pkgs.just
    pkgs.cargo-machete
    pkgs.cargo-audit
    pkgs.cargo-edit
    pkgs.dioxus-cli
    pkgs.lld
    pkgs.sqlx-cli
    pkgs.binaryen
  ];
}
