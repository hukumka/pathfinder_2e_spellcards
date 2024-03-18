{
  description = "Pathfinder 2e SpellCard generator";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, ...}: 
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = {self', pkgs, system, ...}:
        let
          rustVersion = "1.67.0";
          pkgs = import nixpkgs {
            inherit system;
            overlays = [inputs.cargo2nix.overlays.default (import inputs.rust-overlay)];
          };
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            inherit rustVersion;
            packageFun = import ./Cargo.nix;

            packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++ [
              (pkgs.rustBuilder.rustLib.makeOverride {
                name = "yeslogic-fontconfig-sys";
                overrideAttrs = drv: {
                  propagatedNativeBuildInputs = drv.propagatedNativeBuildInputs or [ ] ++ [
                    pkgs.fontconfig.dev
                  ];
                };
              })
            ];
          };
        in {
          packages = rec {
            spellcard_generator = (rustPkgs.workspace.spellcard_generator {}).bin;
            default = spellcard_generator;
          };
          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              pkg-config
              fontconfig.dev
              rust-analyzer-unwrapped
              rust-bin.stable.${rustVersion}.default
            ];
          };
        };
    };
}
