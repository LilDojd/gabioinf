{
  pkgs,
  lib,
  ...
}:

let
  dioxus-cli = pkgs.rustPlatform.buildRustPackage {
    name = "dioxus-cli";
    useFetchCargoVendor = true;
    src = pkgs.fetchFromGitHub {
      owner = "DioxusLabs";
      repo = "dioxus";
      rev = "0dd0f05db09311c26e618e90e60dae72e05d4fa7";
      hash = "sha256-Wkx55o8CT7CBSkmhknT8DwTwsj8+zlJotfK4ElXgcos=";
    };
    buildAndTestSubdir = "packages/cli";
    cargoHash = "sha256-tdAqX17F4vgV8dAOGBl8pMqi1aoucPAEleMOeXTWzAo=";

    checkFlags = [
      "--skip=wasm_bindgen::test::test_cargo_install"
      "--skip=wasm_bindgen::test::test_github_install"
      "--skip=cli::autoformat::test_auto_fmt"
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
    dioxus-cli
    pkgs.wasm-bindgen-cli
    pkgs.lld
    pkgs.tailwindcss
    pkgs.sqlx-cli
    pkgs.binaryen
  ];
}
