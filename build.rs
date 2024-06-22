fn generate_stubs_public(out_path: &std::path::Path, tool: &cc::Tool) {
    // let snappy_cmakelists = std::fs::read_to_string("snappy/CMakeLists.txt")
    //     .expect("Could not find snappy/CMakeLists.txt. Update submodules?");
    // let re = regex::Regex::new(r"project\(Snappy VERSION (\d+).(\d+).(\d+)").expect("Valid regex");
    // let caps = re
    //     .captures(&snappy_cmakelists)
    //     .expect("Can't find snappy version?");
    // let major = caps.get(1).unwrap().as_str();
    // let minor = caps.get(2).unwrap().as_str();
    // let patch = caps.get(3).unwrap().as_str();
    let snappy_stubs_public = std::fs::read_to_string("snappy/snappy-stubs-public.h.in")
        .expect("Could not find snappy/snappy-stubs-public.h.in. Update submodules?")
        .replace(
            "${HAVE_SYS_UIO_H_01}",
            if tool.is_like_msvc() { "0" } else { "1" },
        )
        .replace("${PROJECT_VERSION_MAJOR}", "0") // unused
        .replace("${PROJECT_VERSION_MINOR}", "0") // unused
        .replace("${PROJECT_VERSION_PATCH}", "0") // unused
    ;
    let out_path = out_path.join("snappy-stubs-public.h");
    std::fs::write(&out_path, snappy_stubs_public)
        .unwrap_or_else(|_| panic!("Unable to write {out_path:?}"));
}

fn compile_snappy_cc(dst: &std::path::Path) {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++14")
        .warnings(false)
        .include("snappy")
        .include(dst)
        .file("snappy/snappy-c.cc")
        .file("snappy/snappy-sinksource.cc")
        .file("snappy/snappy-stubs-internal.cc")
        .file("snappy/snappy.cc");

    let tool = build.get_compiler();
    generate_stubs_public(dst, &tool);

    build.compile("snappy");
}

// fn compile_snappy_cmake() {
//     let mut config = cmake::Config::new("snappy");
//     config.define("SNAPPY_BUILD_TESTS", "OFF");
//     config.define("SNAPPY_BUILD_BENCHMARKS", "OFF");
//     let target_features = std::env::var("CARGO_CFG_TARGET_FEATURE")
//         .unwrap_or_default()
//         .split(',')
//         .collect::<Vec<_>>();
//     if target_features.contains(&"avx2") {
//         config.define("SNAPPY_REQUIRE_AVX2", "ON");
//     } else if target_features.contains(&"avx") {
//         config.define("SNAPPY_REQUIRE_AVX", "ON");
//     }
//     let dst = config.build();
//     println!("cargo:rustc-link-search=native={}/lib", dst.display());
// }

#[cfg(feature = "bindgen")]
fn generate_bindings() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header("snappy/snappy-c.h")
        .blocklist_type("max_align_t")
        .blocklist_type("wchar_t")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = manifest_dir.join("bindings.rs");
    bindings
        .write_to_file(&out_path)
        .expect(&format!("Couldn't write bindings to {out_path:?}!"));
}

fn main() {
    // println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=snappy");
    println!("cargo:rustc-link-lib=snappy");

    let dst = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    compile_snappy_cc(&dst);

    #[cfg(feature = "bindgen")]
    generate_bindings();

    std::fs::create_dir_all(dst.join("include")).unwrap();
    std::fs::copy("snappy/snappy-c.h", dst.join("include/snappy-c.h")).unwrap();

    println!("cargo:root={}", dst.to_str().unwrap());
    println!("cargo:include={}/include", dst.to_str().unwrap());
}
