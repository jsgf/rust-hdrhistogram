const HDRHISTO_SRC: &str = "HdrHistogram_c/src";

// From HdrHistogram_c/src/CMakeLists.txt
const HDR_SRCS: &[&str] = &[
    "hdr_encoding.c",
    "hdr_histogram.c",
    #[cfg(feature = "hdr_log")]
    "hdr_histogram_log.c",
    #[cfg(not(feature = "hdr_log"))]
    "hdr_histogram_log_no_op.c",
    "hdr_interval_recorder.c",
    "hdr_thread.c",
    "hdr_time.c",
    "hdr_writer_reader_phaser.c",
];

const HDR_INCLUDES: &[&str] = &[
    "hdr_histogram.h",
    "hdr_histogram_log.h",
    "hdr_interval_recorder.h",
    "hdr_thread.h",
    "hdr_time.h",
    "hdr_writer_reader_phaser.h",
];

const RUST_GLUE: &str = "src/glue.c";
const RUST_GLUE_H: &str = "src/glue.h";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut srcs: Vec<_> = HDR_SRCS
        .iter()
        .map(|src| format!("{}/{}", HDRHISTO_SRC, src))
        .collect();
    srcs.push(RUST_GLUE.to_string());

    let mut headers: Vec<_> = HDR_INCLUDES
        .iter()
        .map(|hdr| format!("{}/{}", HDRHISTO_SRC, hdr))
        .collect();
    headers.push(RUST_GLUE_H.to_string());

    eprintln!("srcs {:?}", srcs);
    eprintln!("headers {:?}", headers);

    cxx_build::bridge("src/lib.rs")
        .files(&srcs)
        .include(HDRHISTO_SRC)
        .include(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
        .cpp(false)
        .compile("hdr_histogram");

    println!("cargo:rustc-link-lib=static=hdr_histogram");
    #[cfg(feature = "hdr_log")]
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=m");

    for s in &srcs {
        println!("cargo:rerun-if-changed={}", s);
    }

    for h in &headers {
        println!("cargo:rerun-if-changed={}", h);
    }

    Ok(())
}
