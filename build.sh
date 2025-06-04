# rasmx86_64 - build.sh
# --------------------
# made by matissoss
# licensed under MPL 2.0

set -e

# constants
_RUST_TARGETS=("x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl")
_EXPT_TARGETS=("-linux-glibc" "-linux-musl")

# $1 = type (--local or --dev; def = --dev)
build(){
	TYPE=$1
	if [[ "$TYPE" == "--dev" ]]; then
		add_instr
		build_dev
	else
		echo "Do you want to install (1) or to build binary (2, default)?"
		CHOICE=$(read)
		add_instr
		if [[ "$CHOICE" == "1" ]]; then
			install
		else
			cargo build --release
		fi
	fi
}
add_instr(){
	echo "Adding instructions to /src/shr/ins_switch.rs..."
	cargo run -- supported-instructions-raw > ins_adder/tmp.txt
	cd ins_adder
	cargo run --release -- tmp.txt
	rm tmp.txt
	mv ins_switch.rs ../src/shr/ins_switch.rs
	cd ..
}
build_dev(){
	rm -rf build
	mkdir build
	for target in "${!_RUST_TARGETS[@]}"; do
		rtt=${_RUST_TARGETS[$target]}
		echo "compiling for ${rtt}..."
		path="${_RASM_BIN}${_EXPT_TARGETS[$target]}"
		cargo build --release --target ${rtt}
		cd build
		tar -czf "${path}.tar.gz" $_RASM_BIN
		rm $_RASM_BIN
		cd ..
	done
}
install(){
	cargo install --path .
}
_test(){
	cd tests
	./test.sh
	cd ..
}

# check for args
if [[ "$1" == "build" ]]; then
	build $2
	exit 0
fi
if [[ "$1" == "refresh" ]]; then
	add_instr
	exit 0
fi
if [[ "$1" == "install" ]]; then
	install
	exit 0
fi
if [[ "$1" == "test" ]]; then
	_test
	exit 0
fi
echo "Options:"
echo "build [[--dev|--local]]"
echo "refresh"
echo "test"
echo "install"
exit 0
