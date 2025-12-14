#!/bin/bash

for day in ./day* ; do
	cd "$day"
	cargo build --release
	cd ..
done

for day in $(ls -d day* | sort -V); do
	echo
	echo "-----"
	echo "$day:"
	"$day/target/release/aoc"
done
