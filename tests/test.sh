#!/bin/bash

set -e

cargo test -- --nocapture

errors=0

for file in ./nasm/*.asm; do
	NASM_FILE=$file
	RASM_FILE="${file/nasm/rasm}"

	nasm $NASM_FILE -o ${NASM_FILE/.asm/.bin} -f bin
	NASM_RES=$(xxd ${NASM_FILE/.asm/.bin})
	cargo run -- -i=$RASM_FILE -o=${RASM_FILE/.asm/.bin} -f=baremetal
	RASM_RES=$(xxd ${RASM_FILE/.asm/.bin})

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
		xxd ${RASM_FILE/.asm/.bin}
		echo "---"
		errors=$((errors+1))
	fi
	rm ${NASM_FILE/.asm/.bin}
	rm ${RASM_FILE/.asm/.bin}
done

if [[ "$errors" == "0" ]]; then
	echo "No errors found!"
	exit 0
else
	echo "$errors error/s were found"
	exit -1
fi
