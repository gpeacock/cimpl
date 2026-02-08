use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(cbindgen::Config::from_file("cbindgen.toml").expect("Failed to load cbindgen.toml"))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/c2pa.h");
    
    println!("cargo:warning=Generated C header at: {}/include/c2pa.h", 
             env::var("CARGO_MANIFEST_DIR").unwrap());
}
