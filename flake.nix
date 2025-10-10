{
  description = "oxwm - A dynamic window manager.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = {
    self,
    nixpkgs,
  }: let
    systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

    forAllSystems = fn: nixpkgs.lib.genAttrs systems (system: fn nixpkgs.legacyPackages.${system});
  in {
    packages = forAllSystems (pkgs: rec {
      default = pkgs.callPackage ./default.nix {};
      oxwm = default;
    });

    devShells = forAllSystems (pkgs: {
      default = pkgs.mkShell {
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
    });

    formatter = forAllSystems (pkgs: pkgs.alejandra);

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
          passthru.providedSessions = ["oxwm"];
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

          environment.systemPackages = [
            cfg.package
            pkgs.rustc
            pkgs.cargo
            pkgs.pkg-config
            pkgs.xorg.libX11
            pkgs.xorg.libXft
            pkgs.xorg.libXrender
            pkgs.freetype
            pkgs.fontconfig
          ];
        };
      };
  };
}
