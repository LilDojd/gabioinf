{
  config,
  pkgs,
  inputs,
  ...
}:
let
  databaseName = "gabioinf";
  wasmTooling = inputs.wasm_bindgen_cli.legacyPackages.${pkgs.stdenv.hostPlatform.system};
in
{
  env.DATABASE_URL = "postgresql://${databaseName}@localhost/${databaseName}?host=${config.env.PGHOST}";

  services.postgres = {
    enable = true;
    initialDatabases = [
      {
        name = databaseName;
        user = databaseName;
      }
    ];
  };

  processes.app = {
    exec = ''secretspec run --scope app -- env DATABASE_URL="$DATABASE_URL" dx serve'';
    after = [ "devenv:processes:postgres" ];
  };

  scripts.prepare-sqlx.exec = ''
    sqlx migrate run
    cargo sqlx prepare -- --all-targets --all-features
  '';

  languages = {
    javascript = {
      enable = true;
      package = pkgs.nodejs_24;
      npm = {
        enable = true;
        install.enable = true;
      };
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
