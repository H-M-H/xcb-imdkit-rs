#!/usr/bin/env sh

set -ex

cd xcb-imdkit
mkdir -p build
cd build
cmake ..
make -j$(nproc)

cp src/xcbimdkit_export.h src/xcb-imdkit_version.h ../../xcb-imdkit-generated-headers
