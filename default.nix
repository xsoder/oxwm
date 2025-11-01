{
  lib,
  rustPlatform,
  pkg-config,
  xorg,
  freetype,
  fontconfig,
}:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "oxwm";
  version = "0.4.0";

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

  postInstall = ''
    install oxwm.desktop -Dt $out/share/xsessions
    install -Dm644 oxwm.1 -t $out/share/man/man1
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
