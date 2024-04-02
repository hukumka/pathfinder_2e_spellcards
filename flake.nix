{
  description = "Pathfinder 2e SpellCard generator";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = {
    self, nixpkgs, flake-utils, rust-overlay, crane
  }: flake-utils.lib.eachDefaultSystem (system: 
    let
      rustVersion = "1.76.0";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      # build time dependencies
      nativeBuildInputs = with pkgs; [
        rustToolchain
        pkg-config
      ];
      # run time dependencies 
      buildInputs = with pkgs; [
        fontconfig.dev
        glib.dev
        gtk4.dev
      ];
      # shell dependencies
      devBuildInputs = with pkgs; [
        rust-analyzer-unwrapped
      ];

      rustToolchain = 
          (pkgs.rust-bin.stable.${rustVersion}.default.override { extensions = [ "rust-src" ]; });
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
      src = craneLib.cleanCargoSource ./.;
      commonArgs = {
        inherit src nativeBuildInputs buildInputs;
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      bin = craneLib.buildPackage (commonArgs // {inherit cargoArtifacts;});
    in with pkgs; {
      packages = {
        inherit bin;
        default = bin;
      };
      devShells.default = mkShell {
        inputsFrom = [bin];
        nativeBuildInputs = devBuildInputs;
      };
    });
}
