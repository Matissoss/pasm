default: 
	@just --list
init: refresh
fmt:
	@cargo fmt
clippy:
	@cargo clippy -q
refresh:
	@echo "invoking ins_adder..."
	@cargo run --package "pasm" -q --features "refresh" -- --supported-instructions-raw > .instructions
	@cargo run --package "ins_adder" -q -- .instructions
	@rm .instructions
	@mv ins_switch.rs src/shr/ins_switch.rs
install_wtests:
	@just refresh
	@just test_wins
	@just install
install:
	cargo install -q --path .
# this might take some time. Requires NASM binary in $PATH
test_full: clean refresh test test_winstructions
test:
	@echo "running clippy..."
	@cargo clippy -q
	@echo "running fmt..."
	@cargo fmt
clean:
	@cargo clean

# Requires NASM binary in $PATH
[working-directory: 'tests']
test_winstructions:
	@./test.sh
