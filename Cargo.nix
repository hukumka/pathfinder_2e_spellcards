# This file was @generated by cargo2nix 0.11.0.
# It is not intended to be manually edited.

args@{
  release ? true,
  rootFeatures ? [
    "spellcard_generator/default"
  ],
  rustPackages,
  buildRustPackages,
  hostPlatform,
  hostPlatformCpu ? null,
  hostPlatformFeatures ? [],
  target ? null,
  codegenOpts ? null,
  profileOpts ? null,
  cargoUnstableFlags ? null,
  rustcLinkFlags ? null,
  rustcBuildFlags ? null,
  mkRustCrate,
  rustLib,
  lib,
  workspaceSrc,
  ignoreLockHash,
}:
let
  nixifiedLockHash = "3ca647eeefafdfe0d94bc1e4671c671b61b39f9f3e8f8c7f4742ab1771eff717";
  workspaceSrc = if args.workspaceSrc == null then ./. else args.workspaceSrc;
  currentLockHash = builtins.hashFile "sha256" (workspaceSrc + /Cargo.lock);
  lockHashIgnored = if ignoreLockHash
                  then builtins.trace "Ignoring lock hash" ignoreLockHash
                  else ignoreLockHash;
in if !lockHashIgnored && (nixifiedLockHash != currentLockHash) then
  throw ("Cargo.nix ${nixifiedLockHash} is out of sync with Cargo.lock ${currentLockHash}")
else let
  inherit (rustLib) fetchCratesIo fetchCrateLocal fetchCrateGit fetchCrateAlternativeRegistry expandFeatures decideProfile genDrvsByProfile;
  profilesByName = {
  };
  rootFeatures' = expandFeatures rootFeatures;
  overridableMkRustCrate = f:
    let
      drvs = genDrvsByProfile profilesByName ({ profile, profileName }: mkRustCrate ({ inherit release profile hostPlatformCpu hostPlatformFeatures target profileOpts codegenOpts cargoUnstableFlags rustcLinkFlags rustcBuildFlags; } // (f profileName)));
    in { compileMode ? null, profileName ? decideProfile compileMode release }:
      let drv = drvs.${profileName}; in if compileMode == null then drv else drv.override { inherit compileMode; };
in
{
  cargo2nixVersion = "0.11.0";
  workspace = {
    spellcard_generator = rustPackages.unknown.spellcard_generator."0.2.0";
  };
  "registry+https://github.com/rust-lang/crates.io-index".adler."1.0.2" = overridableMkRustCrate (profileName: rec {
    name = "adler";
    version = "1.0.2";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "f26201604c87b1e01bd3d98f8d5d9a8fcbb815e8cedb41ffccbeb4bf593a35fe"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".anyhow."1.0.81" = overridableMkRustCrate (profileName: rec {
    name = "anyhow";
    version = "1.0.81";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "0952808a6c2afd1aa8947271f3a60f1a6763c7b912d210184c5149b5cf147247"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "std" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".bstr."1.9.1" = overridableMkRustCrate (profileName: rec {
    name = "bstr";
    version = "1.9.1";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "05efc5cfd9110c8416e471df0e96702d58690178e206e61b7173706673c93706"; };
    features = builtins.concatLists [
      [ "alloc" ]
      [ "default" ]
      [ "std" ]
      [ "unicode" ]
    ];
    dependencies = {
      memchr = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".memchr."2.7.1" { inherit profileName; }).out;
      regex_automata = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".regex-automata."0.4.6" { inherit profileName; }).out;
      serde = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".serde."1.0.197" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".bumpalo."3.15.4" = overridableMkRustCrate (profileName: rec {
    name = "bumpalo";
    version = "3.15.4";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "7ff69b9dd49fd426c69a0db9fc04dd934cdb6645ff000864d98f7e2af8830eaa"; };
    features = builtins.concatLists [
      [ "default" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".cfg-if."1.0.0" = overridableMkRustCrate (profileName: rec {
    name = "cfg-if";
    version = "1.0.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "baf1de4339761588bc0619e3cbc0120ee582ebb74b53b4efbf79117bd2da40fd"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".crc32fast."1.4.0" = overridableMkRustCrate (profileName: rec {
    name = "crc32fast";
    version = "1.4.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "b3855a8a784b474f333699ef2bbca9db2c4a1f6d9088a90a2d25b1eb53111eaa"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "std" ]
    ];
    dependencies = {
      cfg_if = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".cfg-if."1.0.0" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".deranged."0.3.11" = overridableMkRustCrate (profileName: rec {
    name = "deranged";
    version = "0.3.11";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "b42b6fa04a440b495c8b04d0e71b707c585f83cb9cb28cf8cd0d976c315e31b4"; };
    features = builtins.concatLists [
      [ "alloc" ]
      [ "powerfmt" ]
      [ "std" ]
    ];
    dependencies = {
      powerfmt = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".powerfmt."0.2.0" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".encoding_rs."0.8.33" = overridableMkRustCrate (profileName: rec {
    name = "encoding_rs";
    version = "0.8.33";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "7268b386296a025e474d5140678f75d6de9493ae55a5d709eeb9dd08149945e1"; };
    features = builtins.concatLists [
      [ "alloc" ]
      [ "default" ]
    ];
    dependencies = {
      cfg_if = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".cfg-if."1.0.0" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".flate2."1.0.28" = overridableMkRustCrate (profileName: rec {
    name = "flate2";
    version = "1.0.28";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "46303f565772937ffe1d394a4fac6f411c6013172fadde9dcdb1e147a086940e"; };
    features = builtins.concatLists [
      [ "any_impl" ]
      [ "default" ]
      [ "miniz_oxide" ]
      [ "rust_backend" ]
    ];
    dependencies = {
      crc32fast = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".crc32fast."1.4.0" { inherit profileName; }).out;
      miniz_oxide = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".miniz_oxide."0.7.2" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".itoa."1.0.10" = overridableMkRustCrate (profileName: rec {
    name = "itoa";
    version = "1.0.10";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "b1a46d1a171d865aa5f83f92695765caa047a9b4cbae2cbf37dbd613a793fd4c"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".js-sys."0.3.69" = overridableMkRustCrate (profileName: rec {
    name = "js-sys";
    version = "0.3.69";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "29c15563dc2726973df627357ce0c9ddddbea194836909d655df6a75d2cf296d"; };
    dependencies = {
      wasm_bindgen = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen."0.2.92" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".json."0.12.4" = overridableMkRustCrate (profileName: rec {
    name = "json";
    version = "0.12.4";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "078e285eafdfb6c4b434e0d31e8cfcb5115b651496faca5749b88fafd4f23bfd"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".linked-hash-map."0.5.6" = overridableMkRustCrate (profileName: rec {
    name = "linked-hash-map";
    version = "0.5.6";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "0717cef1bc8b636c6e1c1bbdefc09e6322da8a9321966e8928ef80d20f7f770f"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".log."0.4.21" = overridableMkRustCrate (profileName: rec {
    name = "log";
    version = "0.4.21";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "90ed8c1e510134f979dbc4f070f87d4313098b704861a105fe34231c70a3901c"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".lopdf."0.31.0" = overridableMkRustCrate (profileName: rec {
    name = "lopdf";
    version = "0.31.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "07c8e1b6184b1b32ea5f72f572ebdc40e5da1d2921fa469947ff7c480ad1f85a"; };
    features = builtins.concatLists [
      [ "pom" ]
      [ "pom_parser" ]
    ];
    dependencies = {
      encoding_rs = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".encoding_rs."0.8.33" { inherit profileName; }).out;
      flate2 = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".flate2."1.0.28" { inherit profileName; }).out;
      itoa = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".itoa."1.0.10" { inherit profileName; }).out;
      linked_hash_map = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".linked-hash-map."0.5.6" { inherit profileName; }).out;
      log = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".log."0.4.21" { inherit profileName; }).out;
      md5 = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".md5."0.7.0" { inherit profileName; }).out;
      pom = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".pom."3.4.0" { inherit profileName; }).out;
      time = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".time."0.3.34" { inherit profileName; }).out;
      weezl = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".weezl."0.1.8" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".md5."0.7.0" = overridableMkRustCrate (profileName: rec {
    name = "md5";
    version = "0.7.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "490cc448043f947bae3cbee9c203358d62dbee0db12107a74be5c30ccfd09771"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "std" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".memchr."2.7.1" = overridableMkRustCrate (profileName: rec {
    name = "memchr";
    version = "2.7.1";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "523dc4f511e55ab87b694dc30d0f820d60906ef06413f93d4d7a1385599cc149"; };
    features = builtins.concatLists [
      [ "alloc" ]
      [ "std" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".miniz_oxide."0.7.2" = overridableMkRustCrate (profileName: rec {
    name = "miniz_oxide";
    version = "0.7.2";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "9d811f3e15f28568be3407c8e7fdb6514c1cda3cb30683f15b6a1a1dc4ea14a7"; };
    features = builtins.concatLists [
      [ "with-alloc" ]
    ];
    dependencies = {
      adler = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".adler."1.0.2" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".num-conv."0.1.0" = overridableMkRustCrate (profileName: rec {
    name = "num-conv";
    version = "0.1.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "51d515d32fb182ee37cda2ccdcb92950d6a3c2893aa280e540671c2cd0f3b1d9"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".once_cell."1.19.0" = overridableMkRustCrate (profileName: rec {
    name = "once_cell";
    version = "1.19.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "3fdb12b2476b595f9358c5161aa467c2438859caa136dec86c26fdd2efe17b92"; };
    features = builtins.concatLists [
      [ "alloc" ]
      [ "default" ]
      [ "race" ]
      [ "std" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".owned_ttf_parser."0.19.0" = overridableMkRustCrate (profileName: rec {
    name = "owned_ttf_parser";
    version = "0.19.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "706de7e2214113d63a8238d1910463cfce781129a6f263d13fdb09ff64355ba4"; };
    dependencies = {
      ttf_parser = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".ttf-parser."0.19.2" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".pom."3.4.0" = overridableMkRustCrate (profileName: rec {
    name = "pom";
    version = "3.4.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "6c972d8f86e943ad532d0b04e8965a749ad1d18bb981a9c7b3ae72fe7fd7744b"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "utf8" ]
    ];
    dependencies = {
      bstr = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".bstr."1.9.1" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".powerfmt."0.2.0" = overridableMkRustCrate (profileName: rec {
    name = "powerfmt";
    version = "0.2.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "439ee305def115ba05938db6eb1644ff94165c5ab5e9420d1c1bcedbba909391"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".printpdf."0.7.0" = overridableMkRustCrate (profileName: rec {
    name = "printpdf";
    version = "0.7.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "c30a4cc87c3ca9a98f4970db158a7153f8d1ec8076e005751173c57836380b1d"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "js-sys" ]
    ];
    dependencies = {
      ${ if hostPlatform.parsed.cpu.name == "wasm32" && hostPlatform.parsed.kernel.name == "unknown" then "js_sys" else null } = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".js-sys."0.3.69" { inherit profileName; }).out;
      lopdf = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".lopdf."0.31.0" { inherit profileName; }).out;
      owned_ttf_parser = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".owned_ttf_parser."0.19.0" { inherit profileName; }).out;
      time = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".time."0.3.34" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.79" = overridableMkRustCrate (profileName: rec {
    name = "proc-macro2";
    version = "1.0.79";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "e835ff2298f5721608eb1a980ecaee1aef2c132bf95ecc026a11b7bf3c01c02e"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "proc-macro" ]
    ];
    dependencies = {
      unicode_ident = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".unicode-ident."1.0.12" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".quote."1.0.35" = overridableMkRustCrate (profileName: rec {
    name = "quote";
    version = "1.0.35";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "291ec9ab5efd934aaf503a6466c5d5251535d108ee747472c3977cc5acc868ef"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "proc-macro" ]
    ];
    dependencies = {
      proc_macro2 = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.79" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".regex-automata."0.4.6" = overridableMkRustCrate (profileName: rec {
    name = "regex-automata";
    version = "0.4.6";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "86b83b8b9847f9bf95ef68afb0b8e6cdb80f498442f5179a29fad448fcc1eaea"; };
    features = builtins.concatLists [
      [ "dfa-search" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".serde."1.0.197" = overridableMkRustCrate (profileName: rec {
    name = "serde";
    version = "1.0.197";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "3fb1c873e1b9b056a4dc4c0c198b24c3ffa059243875552b2bd0933b1aee4ce2"; };
    features = builtins.concatLists [
      [ "alloc" ]
      [ "std" ]
    ];
    dependencies = {
      ${ if false then "serde_derive" else null } = (buildRustPackages."registry+https://github.com/rust-lang/crates.io-index".serde_derive."1.0.197" { profileName = "__noProfile"; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".serde_derive."1.0.197" = overridableMkRustCrate (profileName: rec {
    name = "serde_derive";
    version = "1.0.197";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "7eb0b34b42edc17f6b7cac84a52a1c5f0e1bb2227e997ca9011ea3dd34e8610b"; };
    features = builtins.concatLists [
      [ "default" ]
    ];
    dependencies = {
      proc_macro2 = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.79" { inherit profileName; }).out;
      quote = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".quote."1.0.35" { inherit profileName; }).out;
      syn = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".syn."2.0.53" { inherit profileName; }).out;
    };
  });
  
  "unknown".spellcard_generator."0.2.0" = overridableMkRustCrate (profileName: rec {
    name = "spellcard_generator";
    version = "0.2.0";
    registry = "unknown";
    src = fetchCrateLocal workspaceSrc;
    dependencies = {
      anyhow = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".anyhow."1.0.81" { inherit profileName; }).out;
      json = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".json."0.12.4" { inherit profileName; }).out;
      printpdf = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".printpdf."0.7.0" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".syn."2.0.53" = overridableMkRustCrate (profileName: rec {
    name = "syn";
    version = "2.0.53";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "7383cd0e49fff4b6b90ca5670bfd3e9d6a733b3f90c686605aa7eec8c4996032"; };
    features = builtins.concatLists [
      [ "clone-impls" ]
      [ "default" ]
      [ "derive" ]
      [ "full" ]
      [ "parsing" ]
      [ "printing" ]
      [ "proc-macro" ]
      [ "quote" ]
      [ "visit" ]
    ];
    dependencies = {
      proc_macro2 = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.79" { inherit profileName; }).out;
      quote = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".quote."1.0.35" { inherit profileName; }).out;
      unicode_ident = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".unicode-ident."1.0.12" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".time."0.3.34" = overridableMkRustCrate (profileName: rec {
    name = "time";
    version = "0.3.34";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "c8248b6521bb14bc45b4067159b9b6ad792e2d6d754d6c41fb50e29fefe38749"; };
    features = builtins.concatLists [
      [ "alloc" ]
      [ "default" ]
      [ "formatting" ]
      [ "parsing" ]
      [ "std" ]
    ];
    dependencies = {
      deranged = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".deranged."0.3.11" { inherit profileName; }).out;
      itoa = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".itoa."1.0.10" { inherit profileName; }).out;
      num_conv = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".num-conv."0.1.0" { inherit profileName; }).out;
      powerfmt = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".powerfmt."0.2.0" { inherit profileName; }).out;
      serde = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".serde."1.0.197" { inherit profileName; }).out;
      time_core = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".time-core."0.1.2" { inherit profileName; }).out;
      time_macros = (buildRustPackages."registry+https://github.com/rust-lang/crates.io-index".time-macros."0.2.17" { profileName = "__noProfile"; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".time-core."0.1.2" = overridableMkRustCrate (profileName: rec {
    name = "time-core";
    version = "0.1.2";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "ef927ca75afb808a4d64dd374f00a2adf8d0fcff8e7b184af886c3c87ec4a3f3"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".time-macros."0.2.17" = overridableMkRustCrate (profileName: rec {
    name = "time-macros";
    version = "0.2.17";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "7ba3a3ef41e6672a2f0f001392bb5dcd3ff0a9992d618ca761a11c3121547774"; };
    features = builtins.concatLists [
      [ "formatting" ]
      [ "parsing" ]
    ];
    dependencies = {
      num_conv = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".num-conv."0.1.0" { inherit profileName; }).out;
      time_core = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".time-core."0.1.2" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".ttf-parser."0.19.2" = overridableMkRustCrate (profileName: rec {
    name = "ttf-parser";
    version = "0.19.2";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "49d64318d8311fc2668e48b63969f4343e0a85c4a109aa8460d6672e364b8bd1"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".unicode-ident."1.0.12" = overridableMkRustCrate (profileName: rec {
    name = "unicode-ident";
    version = "1.0.12";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "3354b9ac3fae1ff6755cb6db53683adb661634f67557942dea4facebec0fee4b"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen."0.2.92" = overridableMkRustCrate (profileName: rec {
    name = "wasm-bindgen";
    version = "0.2.92";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "4be2531df63900aeb2bca0daaaddec08491ee64ceecbee5076636a3b026795a8"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "spans" ]
      [ "std" ]
    ];
    dependencies = {
      cfg_if = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".cfg-if."1.0.0" { inherit profileName; }).out;
      wasm_bindgen_macro = (buildRustPackages."registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-macro."0.2.92" { profileName = "__noProfile"; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-backend."0.2.92" = overridableMkRustCrate (profileName: rec {
    name = "wasm-bindgen-backend";
    version = "0.2.92";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "614d787b966d3989fa7bb98a654e369c762374fd3213d212cfc0251257e747da"; };
    features = builtins.concatLists [
      [ "spans" ]
    ];
    dependencies = {
      bumpalo = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".bumpalo."3.15.4" { inherit profileName; }).out;
      log = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".log."0.4.21" { inherit profileName; }).out;
      once_cell = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".once_cell."1.19.0" { inherit profileName; }).out;
      proc_macro2 = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.79" { inherit profileName; }).out;
      quote = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".quote."1.0.35" { inherit profileName; }).out;
      syn = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".syn."2.0.53" { inherit profileName; }).out;
      wasm_bindgen_shared = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-shared."0.2.92" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-macro."0.2.92" = overridableMkRustCrate (profileName: rec {
    name = "wasm-bindgen-macro";
    version = "0.2.92";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "a1f8823de937b71b9460c0c34e25f3da88250760bec0ebac694b49997550d726"; };
    features = builtins.concatLists [
      [ "spans" ]
    ];
    dependencies = {
      quote = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".quote."1.0.35" { inherit profileName; }).out;
      wasm_bindgen_macro_support = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-macro-support."0.2.92" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-macro-support."0.2.92" = overridableMkRustCrate (profileName: rec {
    name = "wasm-bindgen-macro-support";
    version = "0.2.92";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "e94f17b526d0a461a191c78ea52bbce64071ed5c04c9ffe424dcb38f74171bb7"; };
    features = builtins.concatLists [
      [ "spans" ]
    ];
    dependencies = {
      proc_macro2 = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.79" { inherit profileName; }).out;
      quote = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".quote."1.0.35" { inherit profileName; }).out;
      syn = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".syn."2.0.53" { inherit profileName; }).out;
      wasm_bindgen_backend = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-backend."0.2.92" { inherit profileName; }).out;
      wasm_bindgen_shared = (rustPackages."registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-shared."0.2.92" { inherit profileName; }).out;
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".wasm-bindgen-shared."0.2.92" = overridableMkRustCrate (profileName: rec {
    name = "wasm-bindgen-shared";
    version = "0.2.92";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "af190c94f2773fdb3729c55b007a722abb5384da03bc0986df4c289bf5567e96"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".weezl."0.1.8" = overridableMkRustCrate (profileName: rec {
    name = "weezl";
    version = "0.1.8";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "53a85b86a771b1c87058196170769dd264f66c0782acf1ae6cc51bfd64b39082"; };
    features = builtins.concatLists [
      [ "alloc" ]
      [ "default" ]
      [ "std" ]
    ];
  });
  
}
