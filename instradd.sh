#!/bin/sh

set -e

_=$(cargo --help)

echo "Adding instructions to /src/shr/ins_switch.rs..."
cargo run -- supported-instructions-raw > ins_adder/tmp.txt
cd ins_adder
cargo run --release -- tmp.txt
rm tmp.txt
mv ins_switch.rs ../src/shr/ins_switch.rs
cd ..
