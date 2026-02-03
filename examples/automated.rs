/// Example: Automated DHAT profiling with automatic pprof conversion
///
/// This example shows how to integrate dhat-pprof into your application
/// to automatically convert profiles after execution.

use dhat_pprof::convert;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    // Start DHAT profiling
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::builder()
        .file_name("dhat-heap.json")
        .build();

    println!("Running application with DHAT profiling...");

    // Your application code here
    simulate_work();

    println!("Application finished");

    // DHAT profiler drops here and writes dhat-heap.json

    // Automatically convert to pprof format
    #[cfg(feature = "dhat-heap")]
    {
        println!("Converting DHAT profile to pprof format...");
        if let Err(e) = std::fs::create_dir_all("target/dhat-pprof") {
            eprintln!("Failed to create directory: {}", e);
        } else {
            match convert("dhat-heap.json", "target/dhat-pprof/automated.pb.gz") {
                Ok(_) => {
                    println!("✓ Conversion complete: target/dhat-pprof/automated.pb.gz");
                    println!("\nAnalyze with:");
                    println!("  go tool pprof -http=:8080 target/dhat-pprof/automated.pb.gz");
                }
                Err(e) => eprintln!("Failed to convert profile: {}", e),
            }
        }
    }

    #[cfg(not(feature = "dhat-heap"))]
    println!("Run with: cargo run --example automated --features dhat-heap");
}

fn simulate_work() {
    // Simulate some allocations
    let mut data = Vec::new();
    for i in 0..1000 {
        data.push(allocate_string(i));
    }

    process_data(&data);
}

fn allocate_string(n: usize) -> String {
    format!("Item number {}", n)
}

fn process_data(data: &[String]) {
    let total: usize = data.iter().map(|s| s.len()).sum();
    println!("Processed {} items, total length: {}", data.len(), total);
}
