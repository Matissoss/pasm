<div align=center>
    <h1>extending-pasm.md</h1>
</div>

This is short documentation on how to modify `pasm`'s source code.

## adding new instructions

Adding new instruction is very simple:

- 1. add entry in `src/shr/ins.rs`
- 2. run `./build.sh refresh`
- 3. add entry in `src/pre/chk.rs` (if has different cases for 32-bit and 64-bit add to `check_ins32bit` and `check_ins64bit`, otherwise to `shr_chk`)
- 4. add entry in `src/core/comp.rs` in long `match` switch using `GenAPI` (if instruction has a lot of opcodes, then make own function that returns `GenAPI`)
- 5. create test in `tests/pasm/<INSTRUCTION>.asm` and `tests/nasm/<INSTRUCTION>.asm` (soon will be replaced)
- 6. run `./test.sh` script

### GenAPI

Check `src/core/api.rs` - it is well documented and should be clear.

## adding new features

- 1. specify it's syntax (if it uses closures, modifiers, etc.)
- 2. make module in `src/shr/<MODULE>.rs`
- 3. make tests that can be invoked with `cargo test` (`#[cfg(test)]`)
- 4. either add in `src/pre/tok.rs`, `src/pre/mer.rs:make_operand` or `src/pre/tok.rs:Operand::TryFrom<Token>` or add `[Body/Root]Node` in `src/pre/mer.rs`
- 5. test it if it works as intended

## file header

File header follow format (my favourite one):

```
// PROJ_NAME - PATH
// ----------------
// made by <AUTHOR>
// licensed under <LICENSE>
```

This is one used in `src/core/api.rs`

```
// pasm - src/core/api.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0
```

## documentation style

Title of document should be (if we are using Github flavored markdown (which we are)):

```
<div align=center>
    <h1>FILE.EXTENSION</h1>
</div>

## section
```

(Markdown) Headers should be started with lowercase letter (why? idk) and should use `##` instead of `#`.

Examples should be introduced like:

```
    SOME TEXT
    ```
    EXAMPLE
    ```
```
