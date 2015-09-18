extern crate cmake;

fn main() {
    // Builds the project in the directory located in `libfoo`, installing it
    // into $OUT_DIR
    let dst = cmake::build("HdrHistogram_c/src");

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=hdr_histogram_static");
    println!("cargo:rustc-link-lib=z");
}
