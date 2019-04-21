{ stdenv, fetchgit, cmake, gfortran, python }:

stdenv.mkDerivation {
  name = "dalton-2016";

  src = fetchgit {
    url = "https://gitlab.com/dalton/dalton.git";
    rev = "130ffaa0613bb3af6cac766fc8183d6df7d68917";
    sha256 = "0zwilidbsnvga0m4j8nhwq20jkzv1y69dzc7yll7if8y854afkac";
  };

  hardeningDisable = ["format"];

  buildInputs = [cmake gfortran python];
}
