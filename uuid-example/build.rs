use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR environment variable not set");
    let output_file = PathBuf::from(&crate_dir)
        .join("include")
        .join("cimple_uuid.h")
        .display()
        .to_string();

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=../src/error.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let config = cbindgen::Config::from_file("cbindgen.toml").expect("Couldn't parse config file");

    cbindgen::generate_with_config(&crate_dir, config)
        .expect("Unable to generate bindings")
        .write_to_file(&output_file);

    println!("cargo:warning=Generated C header at: {}", output_file);
}
