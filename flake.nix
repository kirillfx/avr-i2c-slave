{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk/aeb58d5e8faead8980a807c840232697982d47b9";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    ravedude-flake.url = "github:Rahix/avr-hal?dir=ravedude";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, ravedude-flake }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustup rustc cargo rust-analyzer pkgsCross.avr.buildPackages.gcc avrdude udev ];
          buildInputs = [ ravedude-flake.packages.${system}.default ];
          shellHook = ''
            export RAVEDUDE_PORT=/dev/tty.usbserial-0001
          '';
        };
      }
    );
}
