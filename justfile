default: 
	@just --list
refresh:
	@echo "invoking ins_adder..."
	@cargo run --package "pasm" -q --features "refresh" -- --supported-instructions-raw > .instructions
	@cargo run --package "ins_adder" -q -- .instructions
	@rm ins_adder/tmp.txt
	@mv ins_switch.rs src/shr/ins_switch.rs
install:
	cargo install -q --path .
test:
	@echo "running clippy..."
	@cargo clippy -q
	@echo "running fmt..."
	@cargo fmt

# test with instructions
[working-directory: 'tests']
test_wins:
	@./test.sh
