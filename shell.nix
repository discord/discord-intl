let
    nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-24.05";
    pkgs = import nixpkgs { config = {}; overlays = []; };

    fenix = builtins.fetchTarball {
        url = "https://github.com/nix-community/fenix/archive/13b935cb8e697913298acca8309cf031336497f7.tar.gz";
        sha256 = "sha256:0r4mnimn6l5ay9lp4rgqssjilc78qgdvza7wr0w1chfrqdgcsqfw";
    };

    rust-toolchain = fenix.fromToolchainFile {
        file = ../../../rust-toolchain;
        sha256 = "sha256-3fmbQiyGG/DHpEOPwAnCZskyE3MzPUDNCbUnmWZ2h08=";
    };

    pnpm = pkgs.stdenvNoCC.mkDerivation rec {
      pname = "pnpm";
      # pnpm version should match what's used on CI at the very least.
      version = "9.0.6";

      src = pkgs.fetchurl {
        url = "https://registry.npmjs.org/pnpm/-/pnpm-${version}.tgz";
        sha256 = "sha256-BiTjDv+GbN6zY7FQYb23/ZQlsXvBu0LCL19O/eoh9rM=";
      };

      nativeBuildInputs = [
        pkgs.nodejs
      ];
    };
in
    pkgs.mkShell {
        # nativeBuildInputs is usually what you want -- tools you need to run
        nativeBuildInputs = with pkgs.buildPackages; [
            # Node for all of the client packages
            pkgs.nodejs
            # pnpm for managing the workspace
            pnpm
            # Rust for crates
            rust-toolchain
            # Zig for cross compilation
            pkgs.zig
        ];
    }
