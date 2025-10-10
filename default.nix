{ lib
, rustPlatform
, pkg-config
, xorg
, freetype
, fontconfig
}:

rustPlatform.buildRustPackage {
  pname = "oxwm";
  version = "0.1.0";

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

  meta = with lib; {
    description = "A dynamic window manager written in Rust, inspired by dwm";
    homepage = "https://github.com/tonybanters/oxwm";
    license = licenses.gpl3;
    maintainers = [ ];
    platforms = platforms.linux;
    mainProgram = "oxwm";
  };
}
