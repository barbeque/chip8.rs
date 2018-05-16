fn main() {
    // For Mac only, point the linker at /Library/Frameworks
    println!("cargo:rustc-link-search=framework=/Library/Frameworks")
}
