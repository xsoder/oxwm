{
  description = "oxwm - A dynamic window manager.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        packages = {
          default = pkgs.callPackage ./default.nix {};
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
    )
    // {
      nixosModules.default = {
        config,
        lib,
        pkgs,
        ...
      }:
        with lib; let
          cfg = config.services.xserver.windowManager.oxwm;

          oxwmDesktopItem = pkgs.writeTextFile {
            name = "oxwm.desktop";
            destination = "/share/xsessions/oxwm.desktop";
            text = ''
              [Desktop Entry]
              Name=OXWM
              Comment=A dynamic window manager written in Rust
              Exec=${cfg.package}/bin/oxwm
              Type=Application
              DesktopNames=OXWM
            '';
          };
        in {
          options.services.xserver.windowManager.oxwm = {
            enable = mkEnableOption "oxwm window manager";

            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.default;
              description = "The oxwm package to use";
            };
          };

          config = mkIf cfg.enable {
            services.xserver.windowManager.session = [
              {
                name = "oxwm";
                start = ''
                  ${cfg.package}/bin/oxwm &
                  waitPID=$!
                '';
              }
            ];

            services.displayManager.sessionPackages = [oxwmDesktopItem];
            environment.systemPackages = [cfg.package];
          };
        };
    };
}
