extern crate cmake;

use cmake::Config;

fn main() {
    // Builds the project in the directory located in `libfoo`, installing it
    // into $OUT_DIR
    let dst = Config::new("HdrHistogram_c/src")
        .cflag("-std=gnu99")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=hdr_histogram_static");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=m");
}
