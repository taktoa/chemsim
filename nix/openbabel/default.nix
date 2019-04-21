{ stdenv, fetchurl, pkgs }:

with { version = "2.4.1"; };

stdenv.mkDerivation rec {
  name = "openbabel-${version}";

  src = fetchurl {
    url    = "mirror://sourceforge/openbabel/${version}/${name}.tar.gz";
    sha256 = "1z3d6xm70dpfikhwdnbzc66j2l49vq105ch041wivrfz5ic3ch90";
  };

  buildInputs = [ pkgs.cmake pkgs.eigen3_3 ];
}
