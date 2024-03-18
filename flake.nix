{
  description = "Pathfinder 2e SpellCard generator";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, ...}: 
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = {self', pkgs, system, ...}:
        let
          rustVersion = "1.65.0";
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            inherit rustVersion;
            packageFun = ./Cargo.nix;
          };
        in {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [ inputs.cargo2nix.overlays.default (import inputs.rust-overlay) ];
          };
          packages = rec {
            spellcard_generator = (rustPkgs.workscape.spellcard_generator {});
            default = spellcard_generator;
          };
          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              pkg-config
              fontconfig
              rust-analyzer-unwrapped
              rust-bin.stable.${rustVersion}.default
            ];
          };
        };
    };
}
