mod dhat;
mod pprof;

pub use dhat::DhatProfile;
pub use pprof::profiles::Profile;

// Re-export serde_json for convenience
pub use serde_json;

use pprof::profiles::{Function, Line, Location, Sample, ValueType};
use prost::Message;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::Path;

/// Helper struct to manage the string table with deduplication
struct StringTable {
    strings: Vec<String>,
    index_map: HashMap<String, i64>,
}

impl StringTable {
    fn new() -> Self {
        let mut st = StringTable {
            strings: vec![String::new()],  // Index 0 must be empty string
            index_map: HashMap::new(),
        };
        st.index_map.insert(String::new(), 0);
        st
    }

    fn add(&mut self, s: &str) -> i64 {
        if let Some(&idx) = self.index_map.get(s) {
            return idx;
        }
        let idx = self.strings.len() as i64;
        self.strings.push(s.to_string());
        self.index_map.insert(s.to_string(), idx);
        idx
    }

    fn into_vec(self) -> Vec<String> {
        self.strings
    }
}

/// Parse a DHAT frame table entry
/// Format: "0xaddress: function_name (file:line:col)"
/// Returns: (address, function_name, filename, line_number)
fn parse_frame(entry: &str) -> (u64, String, String, i64) {
    // Try to parse the address
    let address = if let Some(addr_end) = entry.find(':') {
        let addr_str = &entry[..addr_end].trim();
        if addr_str.starts_with("0x") {
            u64::from_str_radix(&addr_str[2..], 16).unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    // Try to parse function name and file info
    let rest = if let Some(colon_pos) = entry.find(':') {
        &entry[colon_pos + 1..].trim()
    } else {
        entry
    };

    // Look for parentheses containing file info
    if let Some(paren_start) = rest.find('(') {
        if let Some(paren_end) = rest.find(')') {
            let function_name = rest[..paren_start].trim().to_string();
            let file_info = &rest[paren_start + 1..paren_end];

            // Parse file:line:col
            let parts: Vec<&str> = file_info.split(':').collect();
            let filename = parts.get(0).unwrap_or(&"").to_string();
            let line_number = parts.get(1)
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);

            return (address, function_name, filename, line_number);
        }
    }

    // Fallback: use the whole rest as function name
    (address, rest.to_string(), String::new(), 0)
}

/// Convert a DHAT profile to pprof format
///
/// This function takes a parsed DHAT profile and converts it to the pprof protobuf format.
/// The resulting profile includes four sample types:
/// - `alloc_objects`: Total number of allocations (DHAT's tbk)
/// - `alloc_space`: Total bytes allocated (DHAT's tb)
/// - `inuse_objects`: Peak number of live objects (DHAT's mbk)
/// - `inuse_space`: Peak bytes in use (DHAT's mb)
///
/// # Examples
///
/// ```no_run
/// use dhat_pprof::{DhatProfile, convert_profile};
///
/// let json_content = std::fs::read_to_string("dhat-heap.json").unwrap();
/// let dhat_profile: DhatProfile = serde_json::from_str(&json_content).unwrap();
/// let pprof_profile = convert_profile(dhat_profile).unwrap();
/// ```
pub fn convert_profile(dhat: DhatProfile) -> Result<Profile, Box<dyn Error>> {
    let mut string_table = StringTable::new();

    // Add sample type strings
    let alloc_objects_type = string_table.add("alloc_objects");
    let alloc_space_type = string_table.add("alloc_space");
    let inuse_objects_type = string_table.add("inuse_objects");
    let inuse_space_type = string_table.add("inuse_space");
    let count_unit = string_table.add("count");
    let bytes_unit = string_table.add("bytes");
    let space_type = string_table.add("space");

    // Build sample types
    let sample_types = vec![
        ValueType {
            r#type: alloc_objects_type,
            unit: count_unit,
        },
        ValueType {
            r#type: alloc_space_type,
            unit: bytes_unit,
        },
        ValueType {
            r#type: inuse_objects_type,
            unit: count_unit,
        },
        ValueType {
            r#type: inuse_space_type,
            unit: bytes_unit,
        },
    ];

    // Parse frame table and build Functions and Locations
    let mut functions = Vec::new();
    let mut locations = Vec::new();
    let mut frame_to_location: HashMap<usize, u64> = HashMap::new();

    for (idx, frame_entry) in dhat.ftbl.iter().enumerate() {
        let (address, func_name, filename, line_num) = parse_frame(frame_entry);

        // Create a unique function ID
        let function_id = (idx + 1) as u64;

        // Add strings to string table
        let func_name_idx = string_table.add(&func_name);
        let filename_idx = if !filename.is_empty() {
            string_table.add(&filename)
        } else {
            0
        };

        // Create Function
        let function = Function {
            id: function_id,
            name: func_name_idx,
            system_name: func_name_idx,
            filename: filename_idx,
            start_line: line_num,
        };
        functions.push(function);

        // Create Location
        let location_id = (idx + 1) as u64;
        let location = Location {
            id: location_id,
            mapping_id: 0,  // No mapping info from DHAT
            address,
            line: vec![Line {
                function_id,
                line: line_num,
                column: 0,
            }],
            is_folded: false,
        };
        locations.push(location);

        frame_to_location.insert(idx, location_id);
    }

    // Build samples from profile points
    let mut samples = Vec::new();
    for pp in dhat.pps {
        // Convert frame stack indices to location IDs
        // DHAT fs is root-first, pprof expects leaf-first, so reverse it
        let location_ids: Vec<u64> = pp.fs.iter()
            .rev()
            .filter_map(|&frame_idx| frame_to_location.get(&frame_idx).copied())
            .collect();

        if location_ids.is_empty() {
            continue;  // Skip samples with no valid locations
        }

        // Map DHAT values to pprof values
        // value[0] = tbk (total block count = alloc_objects)
        // value[1] = tb (total bytes = alloc_space)
        // value[2] = mbk (max block count = inuse_objects)
        // value[3] = mb (max bytes = inuse_space)
        let sample = Sample {
            location_id: location_ids,
            value: vec![
                pp.tbk as i64,
                pp.tb as i64,
                pp.mbk as i64,
                pp.mb as i64,
            ],
            label: vec![],
        };
        samples.push(sample);
    }

    // Build the profile
    let profile = Profile {
        sample_type: sample_types,
        sample: samples,
        mapping: vec![],
        location: locations,
        function: functions,
        string_table: string_table.into_vec(),
        drop_frames: 0,
        keep_frames: 0,
        time_nanos: (dhat.tg * 1000) as i64,  // µs to ns
        duration_nanos: ((dhat.te - dhat.tg) * 1000) as i64,  // µs to ns
        period_type: Some(ValueType {
            r#type: space_type,
            unit: bytes_unit,
        }),
        period: 524288,  // default heap sampling rate
        comment: vec![],
        default_sample_type: 0,
        doc_url: 0,
    };

    Ok(profile)
}

/// Write a pprof profile to a gzip-compressed protobuf file
///
/// # Examples
///
/// ```no_run
/// use dhat_pprof::{DhatProfile, convert_profile, write_pprof};
///
/// let json_content = std::fs::read_to_string("dhat-heap.json").unwrap();
/// let dhat_profile: DhatProfile = serde_json::from_str(&json_content).unwrap();
/// let pprof_profile = convert_profile(dhat_profile).unwrap();
/// write_pprof(&pprof_profile, "output.pb.gz").unwrap();
/// ```
pub fn write_pprof<P: AsRef<Path>>(profile: &Profile, output_path: P) -> Result<(), Box<dyn Error>> {
    // Encode to protobuf binary
    let encoded = profile.encode_to_vec();

    // Compress with gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&encoded)?;
    let compressed = encoder.finish()?;

    // Write to file
    std::fs::write(output_path, compressed)?;

    Ok(())
}

/// Convert a DHAT JSON file to a pprof protobuf file
///
/// This is a convenience function that reads a DHAT JSON file, converts it to pprof format,
/// and writes the output as a gzip-compressed protobuf file.
///
/// # Examples
///
/// ```no_run
/// use dhat_pprof::convert;
///
/// convert("dhat-heap.json", "dhat-heap.pb.gz").unwrap();
/// ```
pub fn convert<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<(), Box<dyn Error>> {
    // Read and parse DHAT JSON
    let json_content = std::fs::read_to_string(input_path)?;
    let dhat_profile: DhatProfile = serde_json::from_str(&json_content)?;

    // Convert to pprof
    let pprof_profile = convert_profile(dhat_profile)?;

    // Write output
    write_pprof(&pprof_profile, output_path)?;

    Ok(())
}
