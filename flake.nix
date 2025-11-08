{
  description = "oxwm - A dynamic window manager.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = {
    self,
    nixpkgs,
  }: let
    systems = ["x86_64-linux" "aarch64-linux"];

    forAllSystems = fn: nixpkgs.lib.genAttrs systems (system: fn nixpkgs.legacyPackages.${system});
  in {
    packages = forAllSystems (pkgs: rec {
      default = pkgs.callPackage ./default.nix {
        # use git rev for non tagged releases
        gitRev = self.rev or self.dirtyRev or null;
      };
      oxwm = default;
    });

    devShells = forAllSystems (pkgs: {
      default = pkgs.mkShell {
        buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.alacritty
          pkgs.just
          pkgs.xorg.xorgserver
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
    }: let
      inherit (lib) mkEnableOption mkOption mkIf types;
      cfg = config.services.xserver.windowManager.oxwm;
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
        services.displayManager.sessionPackages = [cfg.package];

        environment.systemPackages = [
          cfg.package
        ];
      };
    };
  };
}
