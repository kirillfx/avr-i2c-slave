{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    ravedude.url = "github:Rahix/avr-hal?dir=ravedude";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, ravedude }:
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
          nativeBuildInputs = with pkgs; [ rustc cargo rust-analyzer avr-gcc avrdude ];
          buildInputs = [ ravedude ];
          shellHook = ''
            export RAVEDUDE_PORT=/dev/tty.usbserial-0001
          '';
        };
      }
    );
}
