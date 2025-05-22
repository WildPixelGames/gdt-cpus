use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let config = cbindgen::Config::from_file("cbindgen.toml").unwrap_or_default();

    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/gdt_cpus/gdt_cpus.h");

    println!("cargo:rerun-if-changed={}", crate_dir);
}
