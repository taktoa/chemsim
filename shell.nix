{ pkgs ? import ./nix/nixpkgs.nix }:

pkgs.stdenv.mkDerivation {
  name = "chemsim-0.1.0";

  src = null;

  buildInputs = [
    pkgs.arrayfire
    pkgs.forge
    pkgs.liblapack
    pkgs.sqlite
    pkgs.nanomsg
    pkgs.capnproto
    pkgs.opencl-headers
    pkgs.opencl-icd
    pkgs.SDL2
    pkgs.glfw
    pkgs.clfft
    pkgs.pkgconfig
    pkgs.libvpx
    pkgs.ffmpeg-full
    pkgs.rustup
    pkgs.cmake
    # pkgs.openbabel
    pkgs.perl
    # (import /nix/store/m5aygxcn0cvnvsk3i66r7b1d8sk2av39-cp2k-2018-07-25.drv)
  ];

  AF_PATH = pkgs.arrayfire;

  LIBCLANG_PATH = "${pkgs.llvmPackages_6.libclang.lib}/lib";

  LD_LIBRARY_PATH = pkgs.stdenv.lib.makeLibraryPath [
    pkgs.xlibs.libX11
    pkgs.xlibs.libXcursor
    pkgs.xlibs.libXxf86vm
    pkgs.xlibs.libXi
    pkgs.xlibs.libXrandr
    pkgs.libGLU_combined
    "/run/opengl-driver"
    "/run/opengl-driver-32"
  ];
}
