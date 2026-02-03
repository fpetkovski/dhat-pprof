use std::process::Command;

fn main() {
    Command::new("protoc")
        .args(&[
            "src/pprof/profile.proto",
            "--prost_out=src/pprof",
            "--prost_opt=compile_well_known_types",
            "--prost_opt=extern_path=.google.protobuf=::pbjson_types",
            "--prost-serde_out=src/pprof",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
