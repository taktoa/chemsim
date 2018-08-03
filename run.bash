#!/usr/bin/env bash

export AF_PATH="/nix/store/l4jraibdbnwpy6yiv5khrlcvj2a85bx9-arrayfire-3.6.1/"

export LD_LIBRARY_PATH="/nix/store/c1giyb1n28nvfpap73c4by4syiw52jv2-libX11-1.6.5/lib:${LD_LIBRARY_PATH}"
export LD_LIBRARY_PATH="/nix/store/875l6l5694s76dagmnfgiw91rbjsr147-libXcursor-1.1.15/lib:${LD_LIBRARY_PATH}"
export LD_LIBRARY_PATH="/nix/store/snmmf2k7cndczw2c0bfw4yaalka5xd34-libXxf86vm-1.1.4/lib:${LD_LIBRARY_PATH}"
export LD_LIBRARY_PATH="/nix/store/a5d226l99z4gb497smnsqavjfpqg7n0f-libXi-1.7.9/lib:${LD_LIBRARY_PATH}"
export LD_LIBRARY_PATH="/nix/store/4j88lngcp80nb1ys3j63qfzqmhabw3nq-libXrandr-1.5.1/lib:${LD_LIBRARY_PATH}"

cargo run "$@"
