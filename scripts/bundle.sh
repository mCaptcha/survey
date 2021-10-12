#!/bin/bash

bench_hash=$(sha256sum ./static/cache/bundle/bench.js \
	| cut -d " " -f 1 \
	| tr  "[:lower:]" "[:upper:]") 

sed -i "s/bench.js/bench.$bench_hash.js/" ./static/cache/bundle/bundle.js
