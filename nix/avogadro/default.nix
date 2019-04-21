{ stdenv, fetchFromGitHub, pkgs }:

stdenv.mkDerivation {
  name = "avogadro-2018-07-06";
  src = fetchFromGitHub {
    owner  = "cryos";
    repo   = "avogadro";
    rev    = "79d22168cb8c5a874189bebf16f7270871f3c469";
    sha256 = "1hd8pnwgs1aiap15pvyifnm98ny4n4bf90xjvffpn3cj2ncvwb9z";
  };

  cmakeFlags = [ "-DUSE_SYSTEM_YAEHMOP=ON" ];
  
  buildInputs = [
    pkgs.cmake
    pkgs.qt48
    pkgs.eigen3_3
    pkgs.pkgconfig
    pkgs.openbabel
    pkgs.boost16x
    pkgs.yaehmop
  ];
}
