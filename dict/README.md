# Project Dict

## Description

`dict` is a Rust-based dictionary implementation featuring a Red-Black Tree for efficient data storage and retrieval. It includes custom implementations for strings and memory allocation to demonstrate low-level Rust capabilities.

The project is designed with C interoperability in mind, providing a `cdylib` target that allows it to be used from C programs.

## Key Features

- **Red-Black Tree**: A balanced binary search tree implementation (`rbtree.rs`) based on the algorithms described in *Introduction to Algorithms* (CLRS).
- **Custom String**: `DictString` implementation for specialized string handling.
- **Custom Allocator**: `DictAllocator` for controlled memory management.
- **C Interoperability**: Examples showing how to use the library from both Rust and C.

## Project Structure

```
.
├── src
│   ├── dict_allocator.rs  # Custom memory allocator
│   ├── dict_string.rs     # Custom string implementation
│   ├── lib.rs             # Library entry point
│   └── rbtree.rs          # Red-Black Tree implementation
├── examples
│   ├── example.c          # Usage example in C
│   └── example.rs         # Usage example in Rust
├── Cargo.toml
└── clippy.toml
```

## How to Run

### Rust Example
To run the Rust example:
```bash
cargo run --example example
```

### C Example
To compile and run the C example (requires `gcc` and a built library):
```bash
cargo build
gcc examples/example.c -L ./target/debug -l dict -I . -o example_c
LD_LIBRARY_PATH=./target/debug ./example_c
```
