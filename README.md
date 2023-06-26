## Building
### macOS
Install libclang:
```
$ brew install llvm
```

Install [rustup](https://rustup.rs/).

Compile:
```
$ cargo build
```

## Running
### macOS
```
$ DYLD_LIBRARY_PATH=/usr/local/opt/llvm/lib/ cargo run -- -c examples/01
```
