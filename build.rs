#[cfg(feature = "runtime")]
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(feature = "runtime")]
const COMMANDS: &[&str] = &[
    "bare_invalidate",
    "bare_init",
    "bare_start_file",
    "bare_start_utf8",
    "bare_start_bytes",
    "bare_read",
    "bare_write",
    "bare_update",
    "bare_suspend",
    "bare_resume",
    "bare_wakeup",
    "bare_terminate",
];

#[cfg(not(feature = "runtime"))]
fn main() {}

#[cfg(feature = "runtime")]
fn main() {
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let install = build_bare_kit(&src, &out).join(match &*target_platform {
        "android" | "linux" => "lib",
        "ios" | "macos" => "Frameworks",
        "windows" => "bin",
        os => panic!("Unsupported platform: {os}"),
    });

    generate_bindings(&src, &out);

    println!("cargo::metadata=INSTALL_DIR={}", install.display());

    tauri_plugin::Builder::new(COMMANDS).build();
}

#[cfg(feature = "runtime")]
fn build_bare_kit<S: AsRef<Path>, O: AsRef<Path>>(src: &S, out: &O) -> PathBuf {
    let src = src.as_ref();
    let out = out.as_ref();
    let build = out.join("build");
    let prefix = out.join("install");

    let target_platform = match &*env::var("CARGO_CFG_TARGET_OS").unwrap() {
        "android" => "android",
        "ios" => "ios",
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "win32",
        os => panic!("Unsupported platform: {os}"),
    };
    let target_arch = match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "arm" => "arm",
        "aarch64" => "arm64",
        "x86" => "ia32",
        "x86_64" => "x64",
        arch => panic!("Unsupported architecture: {arch}"),
    };
    #[cfg(unix)]
    let runner = "npx";
    #[cfg(windows)]
    let runner = "npx.cmd";
    let make = "bare-make";
    let mut args = vec![
        make,
        "generate",
        "--source",
        src.to_str().unwrap(),
        "--build",
        build.to_str().unwrap(),
        "--platform",
        target_platform,
        "--arch",
        target_arch,
    ];

    #[cfg(target_vendor = "apple")]
    if env::var("CARGO_CFG_TARGET_ABI").unwrap() == "sim" {
        args.push("--simulator");
    }

    if env::var("DEBUG").unwrap() == "true" {
        args.push("--debug");
    }

    if target_platform == "android" {
        args.append(&mut vec!["-D", "ANDROID_STL=c++_shared"]);
    }

    #[cfg(windows)]
    args.append(&mut vec!["-D", "CMAKE_OBJECT_PATH_MAX=4096"]);

    assert!(Command::new(runner).args(args).status().unwrap().success());
    assert!(Command::new(runner)
        .args([make, "build", "--build", build.to_str().unwrap(),])
        .status()
        .unwrap()
        .success());
    assert!(Command::new(runner)
        .args([
            make,
            "install",
            "--build",
            build.to_str().unwrap(),
            "--prefix",
            prefix.to_str().unwrap(),
        ])
        .status()
        .unwrap()
        .success());

    prefix
}

#[cfg(feature = "runtime")]
fn generate_bindings<S: AsRef<Path>, O: AsRef<Path>>(src: &S, out: &O) {
    let src = src.as_ref();
    let out = out.as_ref();
    let header = src.join("include/bare-kit.h").canonicalize().unwrap();

    let bindings = bindgen::Builder::default()
        .header(header.to_str().unwrap())
        .allowlist_file(".*bare-kit\\.h")
        .allowlist_file(".*stdbool\\.h")
        .allowlist_file(".*stddef\\.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap();
    bindings.write_to_file(out.join("bindings.rs")).unwrap();
}
