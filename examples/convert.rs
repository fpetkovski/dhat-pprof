use dhat_pprof::{convert, convert_profile, write_pprof, DhatProfile};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Ensure target directory exists
    std::fs::create_dir_all("target/dhat-pprof")?;

    // Method 1: Simple one-liner conversion
    println!("Method 1: Simple conversion");
    convert("examples/dhat-heap.json", "target/dhat-pprof/output1.pb.gz")?;
    println!("  ✓ Converted to target/dhat-pprof/output1.pb.gz\n");

    // Method 2: Convert with custom processing
    println!("Method 2: Convert with custom processing");
    let json_content = std::fs::read_to_string("examples/dhat-heap.json")?;
    let dhat_profile: DhatProfile = dhat_pprof::serde_json::from_str(&json_content)?;

    println!("  Profile stats:");
    println!("    - Profile points: {}", dhat_profile.pps.len());
    println!("    - Functions: {}", dhat_profile.ftbl.len());
    println!("    - Command: {}", dhat_profile.cmd);
    println!("    - Duration: {} µs", dhat_profile.te - dhat_profile.tg);

    let pprof_profile = convert_profile(dhat_profile)?;
    write_pprof(&pprof_profile, "target/dhat-pprof/output2.pb.gz")?;
    println!("  ✓ Converted to target/dhat-pprof/output2.pb.gz\n");

    // Method 3: Access the Profile object for further processing
    println!("Method 3: Access Profile object");
    let json_content = std::fs::read_to_string("examples/dhat-heap.json")?;
    let dhat_profile: DhatProfile = dhat_pprof::serde_json::from_str(&json_content)?;
    let pprof_profile = convert_profile(dhat_profile)?;

    println!("  pprof Profile stats:");
    println!("    - Samples: {}", pprof_profile.sample.len());
    println!("    - Locations: {}", pprof_profile.location.len());
    println!("    - Functions: {}", pprof_profile.function.len());
    println!("    - Sample types: {:?}", pprof_profile.sample_type.len());

    // You can now manipulate the profile before writing
    write_pprof(&pprof_profile, "target/dhat-pprof/output3.pb.gz")?;
    println!("  ✓ Converted to target/dhat-pprof/output3.pb.gz\n");

    println!("All conversions complete!");

    Ok(())
}
