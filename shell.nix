{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  name = "chemsim-0.1.0";

  src = null;

  buildInputs = [
    pkgs.arrayfire
    pkgs.liblapack
    pkgs.sqlite
    pkgs.nanomsg
    pkgs.capnproto
    pkgs.opencl-headers
    pkgs.opencl-icd
    pkgs.SDL2
    pkgs.glfw
    pkgs.clfft
  ];

  AF_PATH = "${pkgs.arrayfire}";

  LD_LIBRARY_PATH = pkgs.stdenv.lib.makeLibraryPath [
    pkgs.xlibs.libX11
    pkgs.xlibs.libXcursor
    pkgs.xlibs.libXxf86vm
    pkgs.xlibs.libXi
    pkgs.xlibs.libXrandr
    "/run/opengl-driver"
    "/run/opengl-driver-32"
  ];
}
