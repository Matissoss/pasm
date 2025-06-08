#!/bin/sh

NASM_BIN="nasm"
RASM_BIN="rasm"
BIN=.tmp/$RASM_BIN
TESTING_PROFILE="testing"
NASM_FLAGS="-Wno-prefix-seg -Wno-prefix-hle -Wno-label-orphan"
RASM_FLAGS="-t"
NASM_FILE_RES=.tmp/nasm-tmp.bin
RASM_FILE_RES=.tmp/rasm-tmp.bin
SXD_BIN="sxd"
SXD_FLAGS="-c -C"

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

#echo "running checks..."
#cargo check -q
#echo "running tests..."
#cargo test -q
#echo "running clippy..."
#cargo clippy -q

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
	
	RASM_RES=$(sxd -1=$RASM_FILE_RES)
	NASM_RES=$(sxd -1=$NASM_FILE_RES)

	if [[ "$RASM_RES" != "$NASM_RES" ]]; then
		printf "\nNASM FILE\n"
		cat $NASM_FILE
		echo   "-------------"
		printf "\nRASM FILE\n"
		cat $RASM_FILE
		printf "\nNASM HEX DUMP\n"
		echo   "-------------"
		sxd -1=$NASM_FILE_RES $SXD_FLAGS
		echo "-------------"
		echo "RASM HEX DUMP"
		echo "-------------"
		sxd -1=$RASM_FILE_RES $SXD_FLAGS
		echo "-------------"
		echo "     DIFF    "
		echo " left = RASM "
		echo "right = NASM"
		echo "-------------"
		sxd -1=$RASM_FILE_RES -2=$NASM_FILE_RES --diff -e -lw=16 $SXD_FLAGS
		echo ""
		echo ""
		errors=$((errors + 1))
	fi

	rm $NASM_FILE_RES
	rm $RASM_FILE_RES
done

# reverts change to set -e
#set -e himBHse
#
#for file in ./elf/*.asm; do
#	rm -f .tmp/tmp.o
#	$BIN -i=$file -o=.tmp/tmp.o -f=elf64 $RASM_FLAGS
#	readelf_res=$(readelf -a ".tmp/tmp.o" | grep -i "error:|warning:")
#	if [[ "$?" != "0" ]]; then
#		_=""
#	fi
#	if [[ $readelf_res != "" ]]; then
#		errors=$((errors+1))
#		echo "Invalid output in ${file}:"
#		readelf -a ".tmp/tmp.o"
#	fi
#	rm .tmp/tmp.o
#done

if [[ "$errors" == "0" ]]; then
	echo "No errors found!"
	exit 0
else
	echo "$errors error/s found"
	exit -1
fi
