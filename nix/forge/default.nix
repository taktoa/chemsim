{ stdenv, fetchFromGitHub, cmake, pkgconfig,
  arrayfire, expat, fontconfig, freeimage, freetype, boost,
  mesa_noglu, libGLU_combined, glfw3, SDL2, cudatoolkit
}:

stdenv.mkDerivation {
  name = "forge-1.0.4";

  src = fetchFromGitHub {
    owner  = "arrayfire";
    repo   = "forge";
    rev    = "650bf611de102a2cc0c32dba7646f8128f0300c8";
    sha256 = "00pmky6kccd7pwi8sma79qpmzr2f9pbn6gym3gyqm64yckw6m484";
    fetchSubmodules = true;
  };

  buildInputs = [
    cmake pkgconfig
    expat
    fontconfig
    freetype
    boost.out
    boost.dev
    freeimage
    mesa_noglu
    libGLU_combined
    glfw3
    SDL2
    cudatoolkit
    arrayfire
  ];
}
