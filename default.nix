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

  cargoSha256 = "18fqlzwd4iqa00zdgnagk5z36hialpjd8gjcdgl0d68pd6ybgmdc";

  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    openssl
    pcsclite
  ];
}
