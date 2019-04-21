{ stdenv, fetchFromGitHub, cmake, fftw, fftwFloat, boost166, opencl-clhpp, ocl-icd }:

stdenv.mkDerivation {
  name = "clFFT-2.12.2";

  src = fetchFromGitHub {
    owner  = "clMathLibraries";
    repo   = "clFFT";
    rev    = "ce107c4d5432d70321af8980a5e7fb64c4b4cce4";
    sha256 = "134vb6214hn00qy84m4djg4hqs6hw19gkp8d0wlq8gb9m3mfx7na";
  };

  postPatch = ''
    cd src
  '';

  buildInputs = [ cmake fftw fftwFloat boost166 opencl-clhpp ocl-icd ];
}
