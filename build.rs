#[cfg(target_os = "macos")]
fn main() {
    // For Mac only, point the linker at /Library/Frameworks
    println!("cargo:rustc-link-search=framework=/Library/Frameworks")
}

#[cfg(not(target_os = "macos"))]
fn main() {

}
