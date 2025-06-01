# Pre-build
echo "Running pre-build script(s)..."
./instradd.sh
echo "Starting installer..."

RASM_BIN="rasmx86_64"

echo "RASM Instalation Script"
echo "-----------------------"

if [[ $1 == "" ]] || [[ $1 == "-h" ]]; then
	echo "flags:"
	echo "  -t  : test"
	echo "  -l  : local install (installs binary inside of ~/.local/bin)"
	echo "  -lu : local install (installs binary inside of ~/.local/bin) without tests"
	echo "  -c  : local install with cargo"
	echo "  -cu : local install with cargo without tests"
	echo "  -d  : dev build (building for ALL supported targets; not recommended)"
	echo "  -h  : help"
fi

if [[ $1 == "-t" ]]; then
	cargo test
	cargo fmt
	cargo clippy
	if [[ $? != 0 ]]; then
		echo "This version of RASM is not functional (errors were found during tests)"
	fi
	cd tests
	./test.sh
	if [[ $? != 0 ]]; then
		echo "This version of RASM is not functional (errors were found during tests)"
	fi
	cd ..
	exit 0
fi

if [[ $1 == "-c" ]]; then
	cargo test
	cargo fmt
	cargo clippy
	if [[ $? != 0 ]]; then
		echo "This version of RASM is not functional (errors were found during tests)"
	fi
	cd tests
	./test.sh
	if [[ $? != 0 ]]; then
		echo "This version of RASM is not functional (errors were found during tests)"
	fi
	cd ..
	cargo install --path .
	exit 0
fi

if [[ $1 == "-cu" ]]; then
	cargo install --path .
	exit 0
fi

if [[ $1 == "-l" ]]; then
	cargo test
	cargo fmt
	cargo clippy
	if [[ $? != 0 ]]; then
		echo "This version of RASM is not functional (errors were found during tests)"
	fi
	cd tests
	./test.sh
	if [[ $? != 0 ]]; then
		echo "This version of RASM is not functional (errors were found during tests)"
	fi
	cd ..
	cargo build --release
	rm -f "${HOME}/.local/bin/${RASM_BIN}"
	mv "target/release/${RASM_BIN}" "${HOME}/.local/bin/${RASM_BIN}"
	exit 0
fi

if [[ $1 == "-lu" ]]; then
	cargo build --release
	rm -f "${HOME}/.local/bin/${RASM_BIN}"
	mv "target/release/${RASM_BIN}" "${HOME}/.local/bin/${RASM_BIN}"
	exit 0
fi

if [[ $1 == "-d" ]]; then
	./build.sh
	exit 0
fi
