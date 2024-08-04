let
    # Using an unreleased version here to get pnpm_9, since it's not released in a stable yet.
    nixpkgs = builtins.fetchTarball "https://github.com/NixOS/nixpkgs/tarball/0e0ab06610ca2a9a266bf7272f818e628059a2d9";
    rust-overlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
    
    pkgs = import nixpkgs {
      config = {};
      overlays = [(import rust-overlay)];
    };

    rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
    pkgs.mkShell {
        # nativeBuildInputs is usually what you want -- tools you need to run
        nativeBuildInputs = with pkgs.buildPackages; [
            # Node for all of the client packages
            pkgs.nodejs_22
            # pnpm for managing the workspace
            pkgs.pnpm_9
            # Rust for crates
            rust-toolchain
            # Zig for cross compilation
            pkgs.zig
        ];
    }
