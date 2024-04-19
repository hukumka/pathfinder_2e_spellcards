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
      rustVersion = "1.77.2";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      inherit (pkgs) lib;
      shareFilter = path: _type: null != builtins.match "^share" (builtins.trace path path);
      sourceFilter = path: type: (shareFilter path type) || (craneLib.filterCargoSources path type);
      # build time dependencies
      nativeBuildInputs = with pkgs; [
        rustToolchain
        pkg-config
        patchelf
        makeWrapper
      ];
      # run time dependencies 
      buildInputs = with pkgs; [
        fontconfig.dev
        glib
        gtk4
      ];
      # shell dependencies
      devBuildInputs = with pkgs; [
        rust-analyzer-unwrapped
      ];

      rustToolchain = 
          (pkgs.rust-bin.stable.${rustVersion}.default.override { extensions = [ "rust-src" ]; });
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
      src = lib.cleanSourceWith { src = ./.; filter = sourceFilter; };
      commonArgs = {
        inherit src nativeBuildInputs buildInputs;
      };
      # GTK requires prebuilt GSettings. XDG_DATA_DIRS is used to provide it with builtin settings.
      gsettings_xdg_dir = "${pkgs.gtk4}/share/gsettings-schemas/gtk4-${pkgs.gtk4.version}";
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      bin = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
        postFixup = ''
          wrapProgram $out/bin/spellcard_generator --prefix XDG_DATA_DIRS : ${gsettings_xdg_dir}/
        '';
      });

    in with pkgs; {
      packages = {
        inherit bin;
        default = bin;
      };
      devShells.default = mkShell {
        inputsFrom = [bin];
        buildInputs = devBuildInputs;
        shellHook = ''
          export XDG_DATA_DIRS="${gsettings_xdg_dir}:$XDG_DATA_DIRS"
        '';
      };
      
    });
}
