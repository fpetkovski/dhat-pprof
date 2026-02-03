# dhat-pprof

Convert [DHAT](https://valgrind.org/docs/manual/dh-manual.html) heap profiling data to [pprof](https://github.com/google/pprof) format.

DHAT is a powerful heap profiler from Valgrind that tracks allocations with high accuracy. This tool converts DHAT's JSON output to the pprof protobuf format, enabling analysis with Google's pprof visualization tools.

## Features

- **4 sample types** mapping DHAT metrics to pprof:
  - `alloc_objects`: Total number of allocations
  - `alloc_space`: Total bytes allocated
  - `inuse_objects`: Peak number of live objects
  - `inuse_space`: Peak memory usage
- **Full call stack preservation** with source file and line information
- **Both library and CLI** for flexible integration
- **Fast conversion** with protobuf encoding and gzip compression

## Installation

### As a CLI tool

```bash
cargo install dhat-pprof
```

### As a library

Add to your `Cargo.toml`:

```toml
[dependencies]
dhat-pprof = "0.1"
```

## CLI Usage

### Basic conversion

```bash
# Convert with default paths (dhat-heap.json → dhat-heap.pb.gz)
dhat-pprof

# Specify input and output paths
dhat-pprof input.json output.pb.gz
```

### Analyze with pprof

```bash
# Text-based top allocations
go tool pprof -top dhat-heap.pb.gz

# Interactive web UI
go tool pprof -http=:8080 dhat-heap.pb.gz

# Focus on specific sample type
go tool pprof -sample_index=alloc_space -top dhat-heap.pb.gz
```

## Library Usage

### Simple conversion

```rust
use dhat_pprof::convert;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    convert("dhat-heap.json", "output.pb.gz")?;
    Ok(())
}
```

### With custom processing

```rust
use dhat_pprof::{DhatProfile, convert_profile, write_pprof};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse DHAT profile
    let json = std::fs::read_to_string("dhat-heap.json")?;
    let dhat_profile: DhatProfile = dhat_pprof::serde_json::from_str(&json)?;

    // Access DHAT metrics
    println!("Command: {}", dhat_profile.cmd);
    println!("Profile points: {}", dhat_profile.pps.len());

    // Convert to pprof
    let pprof_profile = convert_profile(dhat_profile)?;

    // Optionally manipulate the profile here
    println!("Generated {} samples", pprof_profile.sample.len());

    // Write output
    write_pprof(&pprof_profile, "output.pb.gz")?;
    Ok(())
}
```

### Integration with DHAT profiling

```rust
use dhat_pprof::convert;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Your application code here
    run_application();

    // After profiling completes, convert to pprof
    #[cfg(feature = "dhat-heap")]
    convert("dhat-heap.json", "dhat-heap.pb.gz")?;

    Ok(())
}
```

## Workflow Example

### 1. Profile with DHAT

```rust
// Enable DHAT profiling
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Your code here
}
```

Run with DHAT enabled:

```bash
cargo run --features dhat-heap
```

This generates `dhat-heap.json`.

### 2. Convert to pprof

```bash
dhat-pprof dhat-heap.json dhat-heap.pb.gz
```

### 3. Analyze with pprof

```bash
# Interactive web UI (recommended)
go tool pprof -http=:8080 dhat-heap.pb.gz

# Command-line analysis
go tool pprof -top dhat-heap.pb.gz
go tool pprof -list main dhat-heap.pb.gz
```

## Sample Types

The converter maps DHAT metrics to pprof sample types:

| pprof Sample Type | DHAT Field | Description |
|-------------------|------------|-------------|
| `alloc_objects` | `tbk` | Total number of allocations |
| `alloc_space` | `tb` | Total bytes allocated |
| `inuse_objects` | `mbk` | Peak number of live objects |
| `inuse_space` | `mb` | Peak memory usage in bytes |

Use `-sample_index` with pprof to select which metric to analyze:

```bash
go tool pprof -sample_index=0 -top dhat-heap.pb.gz  # alloc_objects
go tool pprof -sample_index=1 -top dhat-heap.pb.gz  # alloc_space
go tool pprof -sample_index=2 -top dhat-heap.pb.gz  # inuse_objects
go tool pprof -sample_index=3 -top dhat-heap.pb.gz  # inuse_space (default)
```

## How It Works

1. **Parse DHAT JSON**: Read the DHAT heap profile with all metrics
2. **Build string table**: Deduplicate all strings (function names, files) into an indexed table
3. **Parse frame table**: Extract function names, file paths, and line numbers from DHAT's stack traces
4. **Create Functions and Locations**: Map each stack frame to pprof Function and Location objects
5. **Generate Samples**: Convert each DHAT profile point to a pprof Sample with 4 values
6. **Encode and compress**: Serialize to protobuf and gzip for efficient storage

## API Documentation

### Functions

#### `convert(input_path, output_path) -> Result<()>`

Convert a DHAT JSON file to pprof format in one call.

#### `convert_profile(dhat_profile: DhatProfile) -> Result<Profile>`

Convert a parsed DHAT profile to a pprof `Profile` object for further processing.

#### `write_pprof(profile: &Profile, output_path) -> Result<()>`

Write a pprof `Profile` to a gzip-compressed protobuf file.

### Types

#### `DhatProfile`

The parsed DHAT JSON structure with fields:
- `pps: Vec<ProfilePoint>` - Profile points (call stacks with metrics)
- `ftbl: Vec<String>` - Frame table (function names and locations)
- `cmd: String` - Command that was profiled
- `tg`, `te: u64` - Start and end timestamps in microseconds

#### `Profile`

The pprof protobuf structure (re-exported from generated code).

## Examples

See the [`examples/`](examples/) directory for more usage patterns:

```bash
# Basic conversion example
cargo run --example convert

# Automated profiling and conversion
cargo run --example automated --features dhat-heap
```

## Benchmarks

Performance benchmarks are available to measure conversion speed:

```bash
cargo bench
```

Typical performance on a modern machine:
- Full end-to-end conversion: ~310 µs
- DHAT JSON parsing: ~22 µs
- Conversion logic: ~80 µs
- Protobuf encoding + gzip: ~209 µs

## Requirements

- Rust 2021 edition or later
- Optional: Go toolchain for pprof analysis

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## See Also

- [DHAT documentation](https://valgrind.org/docs/manual/dh-manual.html)
- [pprof documentation](https://github.com/google/pprof)
- [dhat Rust crate](https://crates.io/crates/dhat)
