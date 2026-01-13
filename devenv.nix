{
  pkgs,
  lib,
  ...
}:

let
  dioxus-cli = pkgs.rustPlatform.buildRustPackage {
    name = "dioxus-cli";
    src = pkgs.fetchFromGitHub {
      owner = "DioxusLabs";
      repo = "dioxus";
      rev = "8f8b58ea80ba0ec8057807bcd58fb609f7a5f2b1";
      hash = "sha256-m4KJ3mchKlhOR45RVf0aGDqRPfRMId5HFnhauw+GHAM=";
    };
    buildAndTestSubdir = "packages/cli";
    cargoHash = "sha256-b7O6uN8zZ1XdEY34GGslIJTcnAGZB6MsOQwi5WCT5YQ=";

    checkFlags = [
      "--skip=wasm_bindgen::test::test_cargo_install"
      "--skip=wasm_bindgen::test::test_github_install"
      "--skip=cli::autoformat::test_auto_fmt"
      "--skip=test_harnesses::run_harness"
    ];

    buildFeatures = [
      "no-downloads"
    ];

    OPENSSL_NO_VENDOR = 1;
    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.cacert
    ];
    buildInputs =
      with pkgs;
      [ openssl ]
      ++ lib.optionals stdenv.isDarwin [
        darwin.apple_sdk.frameworks.CoreServices
      ];

  };
in
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
    pkgs.cargo-nextest
    dioxus-cli
    pkgs.wasm-bindgen-cli
    pkgs.lld
    pkgs.tailwindcss
    pkgs.sqlx-cli
    pkgs.binaryen
  ];
}
