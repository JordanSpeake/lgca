#!/bin/sh
cargo run --release
ffmpeg -y -framerate 30 -i output/density/image%d.png density.mp4
# rm image*.png -f
