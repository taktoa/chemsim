{ stdenv, fetchurl }:

stdenv.mkDerivation {
  name = "yaehmop-3.0.2";

  src = fetchurl {
    url    = "https://github.com/psavery/yaehmop/releases/download/3.0.2/linux64-yaehmop.tgz";
    sha256 = "1m45rrr5qvyqiaq09n2kr5fxfv45a38swjh3wivp52qzzljz3n8k";
  };

  setSourceRoot = ''
    mkdir -v source
    mv -v yaehmop source/yaehmop
    export sourceRoot="source"
  '';
  
  installPhase = ''
    runHook preInstall
    mkdir -pv "$out/bin"
    mv -v yaehmop "$out/bin/yaehmop"
    runHook postInstall
  '';
}
