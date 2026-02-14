{ pkgs, config, ... }:

{
  packages = with pkgs; [
    openssl
    pkg-config
    libxml2
    xmlsec
    libtool # provides libltdl, required by xmlsec at link time
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "rust-src"
    ];
  };

  treefmt = {
    enable = true;
    config.programs = {
      nixpkgs-fmt.enable = true;
      rustfmt.enable = true;
    };
  };

  git-hooks.hooks = {
    treefmt.enable = true;
    clippy = {
      enable = true;
      packageOverrides.cargo = config.languages.rust.toolchainPackage;
      packageOverrides.clippy = config.languages.rust.toolchainPackage;
    };
  };
}
