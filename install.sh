
RASM_BIN="rasmx86_64"

echo "RASM Instalation Script"
echo "-----------------------"

set -e

if [[ $1 == "" ]] || [[ $1 == "-h" ]]; then
	echo "help:"
	echo "	-l : local install (installs binary inside of ~/.local/bin)"
	echo "	-d : dev build (building for ALL supported targets; not recommended)"
	echo " 	-h : help"
fi

if [[ $1 == "-l" ]]; then
	cargo build --release
	rm -f "${HOME}/.local/bin/${RASM_BIN}"
	mv "target/release/${RASM_BIN}" "${HOME}/.local/bin/${RASM_BIN}"
	exit 0
fi

if [[ $1 == "-d" ]]; then
	./build.sh
	exit 0
fi
