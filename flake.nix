{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        toolchain
      ];

      env = {
        RUST_BACKTRACE = "full";
      };

      shellHook = ''
        # Required for use by RustRover, since it doesn't find the toolchain or stdlib by using the PATH
        # RustRover must then be configured to look inside this symlink for the toolchain
        ln --symbolic --force --no-dereference --verbose "${toolchain}" "./.direnv/rust-toolchain"
      '';
    };
  };
}
