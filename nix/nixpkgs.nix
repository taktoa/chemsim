with {
  # rev = "9df3c53f38b35d01f76a7fcd54ad86057517f529";
  # rev = "a8c71037e041725d40fbf2f3047347b6833b1703";
  rev = "07e2b59812de95deeedde95fb6ba22d581d12fbc";

  config = {
    allowUnfree = true;

    packageOverrides = super: let self = super.pkgs; in {
      openbabel = self.callPackage ./openbabel {};
      clfft     = self.callPackage ./clfft     {};
      arrayfire = self.callPackage ./arrayfire {};
      forge     = self.callPackage ./forge     {};
      yaehmop   = self.callPackage ./yaehmop   {};
      avogadro  = self.callPackage ./avogadro  {};
      cp2k      = self.callPackage ./cp2k      {};
      libxsmm   = self.callPackage ./libxsmm   {};
      movipac   = self.callPackage ./movipac   {};
      dalton    = self.callPackage ./dalton    {};
    };
  };

  overlays = [];
};

import (builtins.fetchTarball {
  url    = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  # sha256 = "1p35v6zny71nnl59cnk8pp184l665qa67arl27sfzssp702cwhzn";
  # sha256 = "1z4cchcw7qgjhy0x6mnz7iqvpswc2nfjpdynxc54zpm66khfrjqw";
  sha256 = "1yxmv04v2dywk0a5lxvi9a2rrfq29nw8qsm33nc856impgxadpgf";
}) { inherit config overlays; }
