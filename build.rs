fn main() {
    // Required linker script for esp-hal targets.
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
