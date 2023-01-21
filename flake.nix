{
  description = "Pick emojis using dmenu";
  inputs = { };
  outputs = { self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      definition = { pkgs, languages ? [ ] }:
        pkgs.rustPlatform.buildRustPackage {
          pname = "demoji";
          version = "0.1";
          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = if languages == [ ] then "--all-features" else "--features ${pkgs.lib.concatStringSep "," languages}";
        };

      outputs = with pkgs; {
        packages.${system} = {
          default = self.packages.${system}.demoji;
          demoji = pkgs.callPackage definition { };
        };

        devShells.${system}.default =
          mkShell { nativeBuildInputs = [ xorg.libxcb ]; };
      };

    in outputs;
}
