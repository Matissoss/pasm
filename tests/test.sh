#!/bin/bash

set -e

cargo test -- --nocapture

for file in ./nasm/*.asm; do
	NASM_FILE=$file
	RASM_FILE="${file/nasm/rasm}"
	echo "--- SOURCE CODE ---"
	cat $NASM_FILE
	echo "-------------------"
	cat $RASM_FILE
	echo "-------------------"

	nasm $NASM_FILE -o $NASM_FILE+'.bin' -f bin
	xxd $NASM_FILE+'.bin'
	cargo run -- -i=$RASM_FILE -o=
done
