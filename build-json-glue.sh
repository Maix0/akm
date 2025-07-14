#!/bin/sh

set -e
set -x
cd ./json-glue/
rm -rf ./pkg
rm -rf ../static/js/json-glue
wasm-pack build --target web --release --no-typescript --no-pack
cp -r ./pkg ../static/js/json-glue
