# Pre-build
./instradd.sh

# Tests!
set -e

cd tests
./test.sh
cd ..

_RUST_TARGETS=("x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl")
_EXPT_TARGETS=("-linux-glibc" "-linux-musl")
_RASM_BIN="rasm"

rm -rf .build
mkdir -p .build

for target in "${!_RUST_TARGETS[@]}"; do
	echo "--- ${target} : ${_RUST_TARGETS[$target]} ---"
	path="${_RASM_BIN}${_EXPT_TARGETS[$target]}"

	cargo build --release --target ${_RUST_TARGETS[$target]}
	mv "target/${_RUST_TARGETS[$target]}/release/${_RASM_BIN}" ".build/${_RASM_BIN}"
	cd .build
	tar -czf "${path}.tar.gz" $_RASM_BIN
	rm $_RASM_BIN
	cd ..
done
