{ stdenv, fetchgit, scons, boost, doxygen }:

stdenv.mkDerivation {
  name = "cantera-2.3.0";

  # can't use fetchFromGitHub because there are submodules
  src = fetchgit {
    url    = "https://github.com/Cantera/cantera.git";
    rev    = "8329edf45fc4a3e0b1a93e882be77ef2fbf9c9c5";
    sha256 = "12w6v8lfivf464b4w8gxafwyw5km3y9c4ap5ydhwkngxkj7k6cw6";
  };

  buildInputs = [ scons boost doxygen ];

  buildPhase = ''
    runHook preBuild
    echo "python_package = 'none'"                >> cantera.conf
    echo "boost_inc_dir = '${boost.dev}/include'" >> cantera.conf
    scons -j$NIX_BUILD_CORES -l$NIX_BUILD_CORES build
    scons doxygen
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    scons "prefix=$out" install
    runHook postInstall
  '';
}
