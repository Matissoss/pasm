default: 
	@just --list
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
tested_install:
	@just refresh
	@just test_wins
	@just install
install:
	cargo install -q --path .
test:
	@echo "running clippy..."
	@cargo clippy -q
	@echo "running fmt..."
	@cargo fmt
clean:
	@cargo clean

# test with instructions
[working-directory: 'tests']
test_wins:
	@./test.sh
