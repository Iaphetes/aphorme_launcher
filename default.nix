{ lib
, fetchFromGitHub
, rustPlatform
, fontconfig
, pkg-config
, wayland
, libxkbcommon
, makeWrapper
}:

rustPlatform.buildRustPackage rec {
  pname = "aphorme";
  version = "0.1.18";

  src = fetchFromGitHub {
    owner = "Iaphetes";
    repo = "aphorme_launcher";
    rev = lib.fakeHash;
    hash = lib.fakeHash;
  };

  cargoHash = lib.fakeHash;

  # No tests exist
  doCheck = false;

  libPath = lib.makeLibraryPath [
    libGL
    wayland
    libxkbcommon
  ];

  buildInputs = [ fontconfig libxkbcommon ];
  nativeBuildInputs = [ makeWrapper pkg-config ];

  postInstall = ''
    wrapProgram "$out/bin/aphorme" --prefix LD_LIBRARY_PATH : "${libPath}"
  '';

  meta = with lib; {
    description = "A program launcher for window managers written in rust";
    mainProgram = "aphorme";
    homepage = "https://github.com/Iaphetes/aphorme_launcher";
    license = licenses.gpl3Plus;
    maintainers = with maintainers; [ anytimetraveler ];
    platforms = platforms.linux;
  };
}
