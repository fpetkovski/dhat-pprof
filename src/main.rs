use dhat_pprof::{convert, DhatProfile};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    let input_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "dhat-heap.json".to_string()
    };

    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        "dhat-heap.pb.gz".to_string()
    };

    println!("Reading DHAT profile from: {}", input_path);

    // Read to get stats for display
    let json_content = std::fs::read_to_string(&input_path)?;
    let dhat_profile: DhatProfile = serde_json::from_str(&json_content)?;

    println!("Converting DHAT profile to pprof format...");
    println!("  Profile points: {}", dhat_profile.pps.len());
    println!("  Frame table entries: {}", dhat_profile.ftbl.len());

    // Convert using library function
    convert(&input_path, &output_path)?;

    // Read back to show output stats
    let profile = dhat_pprof::convert_profile(dhat_profile)?;
    println!("  Generated samples: {}", profile.sample.len());
    println!("  Locations: {}", profile.location.len());
    println!("  Functions: {}", profile.function.len());

    println!("Writing pprof profile to: {}", output_path);
    println!("Conversion complete!");
    println!("\nYou can now analyze the profile with:");
    println!("  go tool pprof -top {}", output_path);
    println!("  go tool pprof -http=:8080 {}", output_path);

    Ok(())
}
