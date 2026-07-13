{
  description = "Rust development environment for sqlc-gen-rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              just
              openssl
              pkg-config
              protobuf
              sqlc
            ];
          };
        }
      );

      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
        in
        rec {
          sqlc-gen-rust = rustPlatform.buildRustPackage {
            pname = "sqlc-gen-rust";
            version = "0.1.12";

            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkgs.protobuf ];
            doCheck = false;

            buildPhase = ''
              runHook preBuild
              cargo build \
                --target wasm32-wasip1 \
                --release \
                --offline \
                --package sqlc-gen-rust
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              install -Dm644 \
                target/wasm32-wasip1/release/sqlc-gen-rust.wasm \
                $out/lib/sqlc-gen-rust/sqlc-gen-rust.wasm
              runHook postInstall
            '';
          };

          default = sqlc-gen-rust;
        }
      );

      formatter = forAllSystems (system: (pkgsFor system).nixfmt-tree);
    };
}
