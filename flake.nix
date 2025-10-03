{
  description = "oxwm devshell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

  outputs =
    { self
    , nixpkgs
    ,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
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
          export PKG_CONFIG_PATH="${pkgs.xorg.libX11.dev}/lib/pkgconfig:${pkgs.xorg.libXft.dev}/lib/pkgconfig:${pkgs.xorg.libXrender.dev}/lib/pkgconfig:${pkgs.freetype.dev}/lib/pkgconfig:${pkgs.fontconfig.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

          export PS1="(oxwm-dev) $PS1"
        '';

        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      };

      formatter.${system} = pkgs.alejandra;
    };
}
