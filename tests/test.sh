#!/bin/bash

NASM_BIN="nasm"
PASM_BIN="pasm"
TESTING_PROFILE="testing"
NASM_FLAGS="-Wno-prefix-seg -Wno-prefix-hle -Wno-label-orphan -Wno-prefix-lock"
NASM_FILE_RES="./.tmp/nasm-tmp.bin"
PASM_FILE_RES="./.tmp/pasm-tmp.bin"
SXD_BIN="sxd"
SXD_FLAGS="-c -C"
BIN=./.tmp/$PASM_BIN

_=$($NASM_BIN -h)

if [[ "$?" != "0" ]]; then
	echo "Testing requires having installed NASM binary in PATH!"
	exit
fi
_=$($SXD_BIN -h)

if [[ "$?" != "0" ]]; then
	echo "Testing requires having installed sxd binary in PATH!"
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
mv target/$TESTING_PROFILE/$PASM_BIN tests/.tmp/$PASM_BIN
cd tests
echo "starting tests..."

errors=0

for file in ./nasm/*.asm; do
	NASM_FILE=$file
	PASM_FILE=${file/nasm/pasm}

        echo "Testing file $file..."

	$BIN -i=$PASM_FILE -o=$PASM_FILE_RES -f=bin -t
	$NASM_BIN $NASM_FILE -o $NASM_FILE_RES -f bin $NASM_FLAGS
	
	PASM_RES=$(sxd -1=$PASM_FILE_RES)
	NASM_RES=$(sxd -1=$NASM_FILE_RES)

	if [[ "$PASM_RES" != "$NASM_RES" ]]; then
		printf "\nNASM FILE\n"
		cat $NASM_FILE
		echo   "-------------"
		printf "\nPASM FILE\n"
		cat $PASM_FILE
		printf "\nNASM HEX DUMP\n"
		echo   "-------------"
		sxd -1=$NASM_FILE_RES $SXD_FLAGS
		echo "-------------"
		echo "PASM HEX DUMP"
		echo "-------------"
		sxd -1=$PASM_FILE_RES $SXD_FLAGS
		
		echo "-------------"
		echo "     DIFF    "
		echo " up   = PASM "
		echo "down  = NASM "
		echo "-------------"
		sxd -1=$PASM_FILE_RES -2=$NASM_FILE_RES --diff -e -lw=16 $SXD_FLAGS
		echo ""
		echo ""
		errors=$((errors + 1))
	fi
	rm $NASM_FILE_RES
	rm $PASM_FILE_RES
done

if [[ "$errors" == "0" ]]; then
	echo "No errors found!"
	exit 0
else
	echo "$errors error/s found"
	exit -1
fi
