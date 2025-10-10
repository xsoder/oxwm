{
  description = "oxwm - A dynamic window manager written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages = {
          default = pkgs.callPackage ./default.nix { };
          oxwm = self.packages.${system}.default;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.alacritty
            pkgs.just
            pkgs.xorg.libX11
            pkgs.xorg.libXft
            pkgs.xorg.libXrender
            pkgs.freetype
            pkgs.fontconfig
            pkgs.pkg-config
          ];

          shellHook = ''
            export PS1="(oxwm-dev) $PS1"
          '';

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };

        formatter = pkgs.alejandra;
      }
    ) // {
      # NixOS module
      nixosModules.default = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.services.xserver.windowManager.oxwm;
        in
        {
          options.services.xserver.windowManager.oxwm = {
            enable = mkEnableOption "oxwm window manager";
            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.default;
              description = "The oxwm package to use";
            };
          };

          config = mkIf cfg.enable {
            services.xserver.windowManager.session = [{
              name = "oxwm";
              start = ''
                ${cfg.package}/bin/oxwm &
                waitPID=$!
              '';
            }];

            environment.systemPackages = [ cfg.package ];
          };
        };
    };
}
