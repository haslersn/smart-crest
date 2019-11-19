let
  defaultPkgs = import <nixpkgs> {};
in

{
  openssl ? defaultPkgs.openssl,
  pcsclite ? defaultPkgs.pcsclite,
  pkg-config ? defaultPkgs.pkg-config,
  rustPlatform ? defaultPkgs.rustPlatform
}:

rustPlatform.buildRustPackage rec {
  name = "smart_crest-${version}";
  version = "unstable";

  src = ./.;

  cargoSha256 = "sha256-ZuLnGsnTE3e0rH8bCv3vXH/dRNYjuyLay+lrMw064rU=";

  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    openssl
    pcsclite
  ];
}
