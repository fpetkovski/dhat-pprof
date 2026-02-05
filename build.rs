use std::process::Command;

fn main() {
    // Only run protoc if explicitly requested via environment variable.
    // This avoids requiring protoc for library users since generated files are checked in.
    if std::env::var("DHAT_PPROF_REGENERATE_PROTO").is_ok() {
        println!("cargo:rerun-if-changed=src/pprof/profile.proto");
        println!("cargo:rerun-if-env-changed=DHAT_PPROF_REGENERATE_PROTO");

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
}
