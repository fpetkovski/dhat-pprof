# dhat-pprof

Convert [DHAT](https://valgrind.org/docs/manual/dh-manual.html) heap profiling data to [pprof](https://github.com/google/pprof) format for visualization with Google's pprof tools.

## Features

- **4 sample types**: alloc_objects, alloc_space (default), inuse_objects, inuse_space
- **Full call stack preservation** with source file and line information
- **Fast conversion**: ~310 µs end-to-end
- **Both library and CLI** for flexible integration

## Installation

### CLI tool

```bash
cargo install --git https://github.com/fpetkovski/dhat-pprof
```

### Library

```toml
[dependencies]
dhat-pprof = { git = "https://github.com/fpetkovski/dhat-pprof" }
```

## Quick Start

### 1. Profile with DHAT

```rust
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Your code here
}
```

```bash
cargo run --features dhat-heap  # Generates dhat-heap.json
```

### 2. Convert to pprof

```bash
dhat-pprof dhat-heap.json dhat-heap.pb.gz
```

### 3. Analyze

```bash
go tool pprof -http=:8080 dhat-heap.pb.gz
```

## Library Usage

```rust
use dhat_pprof::convert;

// Simple one-liner
convert("dhat-heap.json", "dhat-heap.pb.gz")?;

// Or with custom processing
let json = std::fs::read_to_string("dhat-heap.json")?;
let dhat_profile: DhatProfile = dhat_pprof::serde_json::from_str(&json)?;
let pprof_profile = dhat_pprof::convert_profile(dhat_profile)?;
dhat_pprof::write_pprof(&pprof_profile, "dhat-heap.pb.gz")?;
```

## Sample Types

| pprof Type              | DHAT Field | Description |
|-------------------------|------------|-------------|
| `alloc_objects`         | `tbk` | Total allocations |
| `alloc_space` (default) | `tb` | Total bytes allocated |
| `inuse_objects`         | `mbk` | Peak live objects |
| `inuse_space`           | `mb` | Peak memory usage |

Select with pprof's `-sample_index`:

```bash
go tool pprof -sample_index=inuse_space -top dhat-heap.pb.gz
```

## Examples

```bash
# Run example programs
cargo run --example convert
cargo run --example automated --features dhat-heap

# Run benchmarks
cargo bench
```

Generated artifacts go to `target/dhat-pprof/` (cleaned by `cargo clean`).

## API Documentation

See inline documentation:

```bash
cargo doc --open
```

Main functions:
- `convert(input, output)` - One-line conversion
- `convert_profile(dhat_profile)` - Returns Profile object for manipulation
- `write_pprof(profile, output)` - Write Profile to file

## License

Licensed under either of Apache-2.0 or MIT at your option.

## Contributing

Contributions welcome! Please submit a Pull Request.

## See Also

- [DHAT documentation](https://valgrind.org/docs/manual/dh-manual.html)
- [pprof documentation](https://github.com/google/pprof)
- [dhat Rust crate](https://crates.io/crates/dhat)
