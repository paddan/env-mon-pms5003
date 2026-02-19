fn main() {
    // Required linker script for xtensa esp-hal targets.
    println!("cargo:rustc-link-arg=-Wl,-Tlinkall.x");
}
