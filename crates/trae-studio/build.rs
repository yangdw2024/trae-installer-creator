use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dir = PathBuf::from(&manifest_dir);
        let manifest_path = dir.join("app.manifest");
        let res_path = dir.join("..").join("..").join("assets").join("forgepack.res");
        println!("cargo:rerun-if-changed={}", res_path.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
        if res_path.is_file() {
            println!("cargo:rustc-link-arg={}", res_path.display());
        }
    }
}
