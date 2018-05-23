#!/usr/bin/env bash

while inotifywait -e modify -r src/;
do
    nice -n 19 cargo test;
done
