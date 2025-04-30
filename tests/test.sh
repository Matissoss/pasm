#!/bin/bash

set -e

cargo test -- --nocapture

errors=0

for file in ./nasm/*.asm; do
	NASM_FILE=$file
	RASM_FILE=${file/nasm/rasm}

	NASM_FILE_RES=${NASM_FILE/.asm/.bin}
	RASM_FILE_RES=${RASM_FILE/.asm/.bin}

	nasm $NASM_FILE -o $NASM_FILE_RES -f bin
	cargo run -- -i=$RASM_FILE -o=$RASM_FILE_RES -f=bin
	
	NASM_RES=$(xxd $NASM_FILE_RES)
	RASM_RES=$(xxd $RASM_FILE_RES)

	if [[ "$NASM_RES" != "$RASM_RES" ]]; then
		echo ""
		echo "--- SOURCE CODE ---"
		cat $NASM_FILE
		echo "-------------------"
		cat $RASM_FILE
		echo "-------------------"
		echo "$NASM_FILE | $RASM_FILE"
		echo "NASM HEX DUMP: "
		xxd ${NASM_FILE/.asm/.bin}
		echo "---"
		echo "RASM HEX DUMP: "
		xxd ${RASM_FILE/.asm/.bin}
		echo "---"
		errors=$((errors+1))
	fi
	rm $NASM_FILE_RES
	rm $RASM_FILE_RES
done

for file in ./elf/*.asm; do
	rm -f main.o
	cargo r -- -i=$file -o=main -f=elf64
	readelf_res=$(readelf -a "main" | grep -i "error|warning" || true)
	if [[ $readelf_res != "" ]]; then
		errors=$((errors+1))
		echo "Invalid output in ${file}:"
		readelf -a "main"
	fi
	rm main
done

if [[ "$errors" == "0" ]]; then
	echo "No errors found!"
	exit 0
else
	echo "$errors error/s were found"
	exit -1
fi
