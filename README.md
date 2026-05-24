# Rust Laboratories

This repository contains a collection of Rust projects and laboratory exercises completed as part of the Rust programming course at the Warsaw University of Technology. 
It demonstrates a progression from basic language features to advanced concepts like custom memory management, concurrency, and C interoperability.

## 🚀 Featured Projects

### 📖 [Project Dict](./dict)
A low-level dictionary implementation in Rust.
- **Data Structure**: Custom Red-Black Tree based on CLRS for efficient $O(\log n)$ operations.
- **Memory Management**: Custom string and allocator implementations.
- **C Interoperability**: Compiled as a `cdylib` with examples showing usage from C.

### 🗄️ [Project Memoria](./memoria)
A lightweight in-memory database with a custom query language.
- **Parser**: Uses the `pest` crate for grammar definition and parsing.
- **CLI**: Interactive command-line interface for database operations.
- **Persistence**: Supports command history and script execution from files.

---

## 🧪 Laboratory Exercises

The repository includes several laboratory tasks, each focusing on specific Rust features:

- **[l_1](./l_1)**: Basics &ndash; random number generation, file I/O, and control flow.
- **[l_2](./l_2)**: Data structures &ndash; structs, traits (`Clone`, `Debug`, `Default`), and ownership basics.
- **[l_3](./l_3)**: Enums &ndash; pattern matching and `to_string` implementation for complex types.
- **[l_4](./l_4)**: Networking &ndash; TCP server implementation and basic network communication (includes **[l_4_helper](./l_4_helper)**).
- **[l_5](./l_5)**: Abstraction &ndash; traits for expressions and statements, building a simple interpreter.
- **[l_7](./l_7)**: Functional programming &ndash; closures, `FnMut`, and advanced iterator usage.
- **[l_8](./l_8)**: Smart pointers &ndash; practical use of `Rc`, `RefCell`, `Cow`, and `LazyCell`.
- **[l_9](./l_9)**: Advanced features &ndash; Generics, Dynamic Dispatch (`dyn`), `Any` type, and Concurrency (threads, scopes, mutexes, and channels).

---

## 🛠️ Getting Started

To explore any of the projects or laboratories, navigate to their respective directories and use standard Cargo commands:

```bash
# To run a laboratory (if it has a main.rs)
cargo run

# To run tests
cargo test

# To run examples (specifically for Project Dict)
cargo run --example example
```
