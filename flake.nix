{
  description = "oxwm devshell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = [
        pkgs.rustc
        pkgs.cargo
        pkgs.xorg.xclock
        pkgs.xterm
        pkgs.alacritty
        pkgs.just
      ];
      shellHook = ''
        export PS1="(oxwm-dev) $PS1"
      '';

      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    };

    formatter.${system} = pkgs.alejandra;
  };
}
