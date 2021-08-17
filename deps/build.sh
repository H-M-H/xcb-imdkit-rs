#!/usr/bin/env sh

set -ex

if [ ! -d "xcb-imdkit" ]; then
  git clone --depth 1 -b 1.0.3 "https://github.com/fcitx/xcb-imdkit"
fi

mkdir -p dist
cd xcb-imdkit
sed -i -e 's/add_library(xcb-imdkit SHARED/add_library(xcb-imdkit STATIC/' "src/CMakeLists.txt"
mkdir -p build
cd build
cmake .. -DCMAKE_INSTALL_PREFIX="../../dist/"
make -j$(nproc)
make install
