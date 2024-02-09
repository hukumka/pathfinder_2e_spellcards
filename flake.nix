{
  description = "Pathfinder 2e SpellCard generator";

  inputs = {
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix.url = "github:nix-community/crate2nix";
    devshell.url = "github:numtide/devshell";
  };

  outputs =
    inputs @ { self
    , nixpkgs
    , flake-parts
    , rust-overlay
    , crate2nix
    , ...
    }: flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devshell.flakeModule
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem = { system, pkgs, lib, inputs', ... }:
        let
          cargoNix = inputs.crate2nix.tools.${system}.appliedCargoNix {
            name = "rustnix";
            src = ./.;
          };
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        rec {
          checks = {
            rustnix = cargoNix.rootCrate.build.override {
              runTests = true;
            };
          };

          packages = {
            rustnix = cargoNix.rootCrate.build;
            default = packages.rustnix;
          };
         
          devshells.default = {
            imports = [
              "${inputs.devshell}/extra/language/c.nix"
            ];
            packages = with pkgs; [
              rust-analyzer-unwrapped
              rust-bin.stable.latest.default
            ];
          };
        };
    };
}
