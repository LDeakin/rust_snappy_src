fn get_snappy_version() -> (u32, u32, u32) {
    let version = env!("CARGO_PKG_VERSION");
    let versions = version.split("+snappy.").collect::<Vec<_>>();
    let snappy_version = match &versions[..] {
        &[_crate_version, snappy_version] => snappy_version,
        _ => panic!("Could not identify snappy version from crate version"),
    };
    let snappy_versions = snappy_version.split(".").collect::<Vec<_>>();
    match &snappy_versions[..] {
        &[major, minor, patch] => (
            major
                .parse::<u32>()
                .expect("Snappy major version is not an integer"),
            minor
                .parse::<u32>()
                .expect("Snappy minor version is not an integer"),
            patch
                .parse::<u32>()
                .expect("Snappy patch version is not an integer"),
        ),
        _ => panic!("Could not identify snappy version from crate version"),
    }
}

fn generate_stubs_public(out_path: &std::path::Path) {
    let (snappy_ver_major, snappy_ver_minor, snappy_ver_patch) = get_snappy_version();
    let unix = std::env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("unix");
    let snappy_stubs_public = std::fs::read_to_string("snappy/snappy-stubs-public.h.in")
        .expect("Could not find snappy/snappy-stubs-public.h.in. Update submodules?")
        .replace("${HAVE_SYS_UIO_H_01}", if unix { "1" } else { "0" })
        .replace("${PROJECT_VERSION_MAJOR}", &snappy_ver_major.to_string())
        .replace("${PROJECT_VERSION_MINOR}", &snappy_ver_minor.to_string())
        .replace("${PROJECT_VERSION_PATCH}", &snappy_ver_patch.to_string());
    let out_path = out_path.join("snappy-stubs-public.h");
    std::fs::write(&out_path, snappy_stubs_public)
        .unwrap_or_else(|_| panic!("Unable to write {out_path:?}"));
}

fn compile_snappy_cc(dst: &std::path::Path) {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++14")
        .include("snappy")
        .include(dst)
        .file("snappy/snappy-c.cc")
        .file("snappy/snappy-sinksource.cc")
        .file("snappy/snappy-stubs-internal.cc")
        .file("snappy/snappy.cc");

    generate_stubs_public(dst);

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
    // `snappy_status` is a plain C enum, so its underlying type is signed under
    // the MSVC ABI and unsigned under the Itanium C++ ABI. `bindings.rs` is
    // shared by every target, so swap whichever typedef bindgen emitted for the
    // host target for a `cfg` that covers both.
    const MSVC: &str = "pub type snappy_status = ::std::os::raw::c_int;";
    const ITANIUM: &str = "pub type snappy_status = ::std::os::raw::c_uint;";

    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header("snappy/snappy-c.h")
        .allowlist_file(r".*snappy-c\.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .rust_target(unsafe { bindgen::RustTarget::stable(63, 0).unwrap_unchecked() })
        .generate()
        .expect("Unable to generate bindings")
        .to_string();

    let cfg = format!(
        "#[cfg(target_env = \"msvc\")]\n{MSVC}\n#[cfg(not(target_env = \"msvc\"))]\n{ITANIUM}"
    );
    let bindings = if bindings.contains(MSVC) {
        bindings.replace(MSVC, &cfg)
    } else if bindings.contains(ITANIUM) {
        bindings.replace(ITANIUM, &cfg)
    } else {
        panic!("`snappy_status` typedef not found, the handling in build.rs is stale")
    };

    let out_path = manifest_dir.join("bindings.rs");
    std::fs::write(&out_path, bindings)
        .unwrap_or_else(|_| panic!("Couldn't write bindings to {out_path:?}!"));
}

fn main() {
    // println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=snappy");
    println!("cargo:rustc-link-lib=static=snappy");

    let dst = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    compile_snappy_cc(&dst);

    #[cfg(feature = "bindgen")]
    generate_bindings();

    std::fs::create_dir_all(dst.join("include")).unwrap();
    std::fs::copy("snappy/snappy-c.h", dst.join("include/snappy-c.h")).unwrap();

    println!("cargo:root={}", dst.to_str().unwrap());
    println!("cargo:include={}/include", dst.to_str().unwrap());
}
