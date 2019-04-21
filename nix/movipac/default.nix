{ stdenv, lib, fetchurl, gfortran, dalton }:

with {
  execPath = lib.makeBinPath [
    dalton
  ];
};

stdenv.mkDerivation {
  name = "movipac-1.0.1";

  src = fetchurl {
    url = "https://www.ethz.ch/content/dam/ethz/special-interest/chab/physical-chemistry/reiher-dam/documents/Software/movipac-1.0.1.tar.bz2";
    sha256 = "0b672axccvi524z1rm6w4icw48jnlwv1spk97qc5vjky7z9fgari";
  };

  hardeningDisable = ["format"];

  buildInputs = [ gfortran ];

  postInstall = ''
    for file in $out/bin/*; do
      wrapProgram "$file" --prefix PATH ${execPath}
    done
  '';
}
