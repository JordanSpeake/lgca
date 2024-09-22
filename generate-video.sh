#!/bin/sh
# rm image*.png -f
cargo run --release
ffmpeg -y -framerate 60 -i image%d.png output.mp4
rm image*.png -f
