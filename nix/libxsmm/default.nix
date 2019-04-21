{ stdenv, fetchFromGitHub, pkgs }:

stdenv.mkDerivation {
  name = "libxsmm-1.9";

  src = fetchFromGitHub {
    owner  = "hfp";
    repo   = "libxsmm";
    rev    = "cd98a1940bfb37d3aaee009cd72b931f09d659eb";
    sha256 = "002scxp0d4kyzq39ncb9lfqc9bn82lrmf8vd5r220f3cwlvzr731";
  };

  enableParallelBuilding = true;

  buildInputs = [
    pkgs.gfortran7
    pkgs.which
    pkgs.gnused
    pkgs.utillinux
    pkgs.coreutils
    pkgs.python27
  ];

  prePatch = ''
    patchShebangs .
  '';

  installPhase = ''
    runHook preInstall
    make install SHELL=${pkgs.bash}/bin/bash PREFIX=$out
    runHook postInstall
  '';
}
