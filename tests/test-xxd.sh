#!/bin/sh

# This is legacy testing script - use ./test.sh instead

NASM_BIN="nasm"
RASM_BIN="rasm"
BIN=.tmp/$RASM_BIN
TESTING_PROFILE="testing"
NASM_FLAGS="-Wno-prefix-seg -Wno-prefix-hle -Wno-label-orphan"
RASM_FLAGS="-t"
NASM_FILE_RES=.tmp/nasm-tmp.bin
RASM_FILE_RES=.tmp/rasm-tmp.bin

_=$($NASM_BIN -h)

if [[ "$?" != "0" ]]; then
	echo "Testing requires having installed NASM binary in PATH!"
	exit
fi

set -e

echo "running checks..."
cargo check -q
echo "running tests..."
cargo test -q
echo "running clippy..."
cargo clippy -q

rm -rf .tmp
mkdir .tmp

cd ..
echo "building tmp binary..."
cargo build --profile $TESTING_PROFILE -q
mv target/$TESTING_PROFILE/$RASM_BIN tests/.tmp/$RASM_BIN
cd tests

errors=0

for file in ./nasm/*.asm; do
	NASM_FILE=$file
	RASM_FILE=${file/nasm/rasm}

	./.tmp/rasm -i=$RASM_FILE -o=$RASM_FILE_RES -f=bin -t
	$NASM_BIN $NASM_FILE -o $NASM_FILE_RES -f bin $NASM_FLAGS
	
	RASM_RES=$(xxd $RASM_FILE_RES)
	NASM_RES=$(xxd $NASM_FILE_RES)

	if [[ "$RASM_RES" != "$NASM_RES" ]]; then
		echo "NASM HEX DUMP"
		echo "-------------"
		xxd $NASM_FILE_RES
		echo "-------------"
		echo "RASM HEX DUMP"
		echo "-------------"
		xxd $RASM_FILE_RES
		echo "-------------"
		errors=$((errors + 1))
	fi

	rm $NASM_FILE_RES
	rm $RASM_FILE_RES
done

if [[ "$errors" == "0" ]]; then
	echo "No errors found!"
	exit 0
else
	echo "$errors error/s were found"
	exit -1
fi
