{
  lib,
  rustPlatform,
  pkg-config,
  xorg,
  freetype,
  fontconfig,
  makeDesktopItem,
}:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "oxwm";
  version = "0.1.12";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    xorg.libX11
    xorg.libXft
    xorg.libXrender
    freetype
    fontconfig
  ];

  postInstall = let
    oxwmDesktopItem = makeDesktopItem rec {
      name = finalAttrs.pname;
      exec = name;
      desktopName = name;
      comment = finalAttrs.meta.description;
    };
  in ''
    install -Dt $out/share/xsessions ${oxwmDesktopItem}/share/applications/oxwm.desktop
  '';

  passthru.providedSessions = ["oxwm"];

  meta = with lib; {
    description = "A dynamic window manager written in Rust, inspired by dwm";
    homepage = "https://github.com/tonybanters/oxwm";
    license = licenses.gpl3;
    platforms = platforms.linux;
    mainProgram = "oxwm";
  };
})
