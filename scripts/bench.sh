#!/bin/bash

N=100
K=3
L=(160 200 240 280 320 360 400 440 480 520 560 600 640 680 720 760 800 840 880 920 960 1000)

for l in ${L[@]}; do
    cargo run --bin random -- -n $N -k $K -l $l -r 100
done
