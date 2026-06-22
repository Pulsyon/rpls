{
  description = "Nix build and run support for rpls";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;
      toolchainFile = ../toolchain.toml;
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          inherit (pkgs) lib;
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile toolchainFile;
          runtimeInputs = [
            pkgs.pkg-config
            rustToolchain
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.apple-sdk_15
          ] ++ [
            pkgs.openssl
          ];
          setupWorkspace = ''
            workspace_root="''${RPLS_WORKSPACE_ROOT:-$PWD}"

            if [ ! -f "$workspace_root/Cargo.toml" ]; then
              echo "rpls wrapper: run from the reth-pulse checkout or set RPLS_WORKSPACE_ROOT" >&2
              exit 1
            fi

            export CARGO_TARGET_DIR="''${CARGO_TARGET_DIR:-$workspace_root/target/nix-run}"
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig''${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
          '';
          mkRunWrapper =
            name: profileArgs:
            pkgs.writeShellApplication {
              inherit name runtimeInputs;
              text = ''
                ${setupWorkspace}

                exec cargo run ${profileArgs} --manifest-path "$workspace_root/Cargo.toml" --bin rpls -- "$@"
              '';
            };
          mkBuildWrapper =
            name: profileArgs:
            pkgs.writeShellApplication {
              inherit name runtimeInputs;
              text = ''
                ${setupWorkspace}

                exec cargo build ${profileArgs} --manifest-path "$workspace_root/Cargo.toml" --bin rpls "$@"
              '';
            };
        in
        {
          default = mkRunWrapper "rpls" "--release";
          debug = mkRunWrapper "rpls-debug" "";
          build-debug = mkBuildWrapper "rpls-build-debug" "";
          build-release = mkBuildWrapper "rpls-build-release" "--release";
        }
      );

      apps = forAllSystems (
        system:
        let
          mkApp = package: programName: {
            type = "app";
            program = "${package}/bin/${programName}";
          };
        in
        {
          default = mkApp self.packages.${system}.default "rpls";
          debug = mkApp self.packages.${system}.debug "rpls-debug";
          release = mkApp self.packages.${system}.release "rpls-release";
          build-debug = mkApp self.packages.${system}.build-debug "rpls-build-debug";
          build-release = mkApp self.packages.${system}.build-release "rpls-build-release";
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile toolchainFile;
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo-deny
              pkgs.openssl
              pkgs.pkg-config
              rustToolchain
            ];
          };
        }
      );
    };
}
