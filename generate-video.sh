#!/bin/sh
cargo run --release
ffmpeg -y -framerate 30 -i image%d.png output.mp4
rm image*.png
