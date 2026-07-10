{
  description = "URL replacement and analytics service";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-26.05;
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, ... }@inputs:
    let
      system = "x86_64-linux";
      overlays = [ rust-overlay.overlays.default ];
      pkgs = import nixpkgs { inherit system overlays; };
      rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      packages.${system} = rec {
        default = linkletter;
        linkletter = pkgs.rustPlatform.buildRustPackage {
          pname = "linkletter";
          version = "0.1.0";

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          meta = {
            description = "URL replacement and analytics service";
            homepage = "https://github.com/mrbjarksen/linkletter";
            licence = nixpkgs.lib.licences.gpl2Only;
            maintainers = with nixpkgs.lib.maintainers; [ mrbjarksen ];
          };
        };
      };

      devShells.${system}.default = pkgs.mkShell rec {
        packages = [ rust ];
      };
    };
}
