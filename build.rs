use std::env;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("windows-gnu") {
        let out_dir = env::var("OUT_DIR").unwrap();
        let resource_file = format!("{}/resource.o", out_dir);

        let status = Command::new("windres")
            .args(&["resource.rc", "-O", "coff", "-o", &resource_file])
            .status()
            .expect("Failed to execute windres. Make sure MinGW-w64 bin folder is in your PATH.");

        if !status.success() {
            panic!("windres failed with status: {}", status);
        }

        println!("cargo:rustc-link-arg={}", resource_file);
        println!("cargo:rerun-if-changed=resource.rc");
        println!("cargo:rerun-if-changed=app.manifest");
    }
}
