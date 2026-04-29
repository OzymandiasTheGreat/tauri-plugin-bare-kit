use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(unix)]
const NPM: &str = "npm";
#[cfg(windows)]
const NPM: &str = "npm.cmd";
#[cfg(unix)]
const RUNNER: &str = "npx";
#[cfg(windows)]
const RUNNER: &str = "npx.cmd";
const MAKE: &str = "bare-make@latest";

#[derive(Debug, Deserialize)]
struct META {
    root: String,
}

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

fn main() {
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let meta_str = String::from_utf8(
        Command::new(env::var("CARGO").unwrap())
            .current_dir(&out)
            .arg("locate-project")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let meta_json: META = serde_json::from_str(&meta_str).unwrap();
    let tauri_root = PathBuf::from(meta_json.root);
    let tauri_root = tauri_root.parent().unwrap();
    let project_root = tauri_root.parent().unwrap();
    let mut extra_args: Vec<&str> = vec![];
    let parent_project_path = format!("PARENT_PROJECT_PATH={}", project_root.display());
    let node_modules_path = format!("NODE_MODULES_PATH={}", out.join("node_modules").display());

    if project_root.join("bare").is_dir() || project_root.join("src-bare").is_dir() {
        extra_args.append(&mut vec!["--define", &*parent_project_path]);
    } else {
        extra_args.append(&mut vec!["--define", &*node_modules_path]);
        fs::copy(src.join("package.json"), out.join("package.json")).unwrap();
        fs::copy(src.join("package-lock.json"), out.join("package-lock.json")).unwrap();
        assert!(Command::new(NPM)
            .args([
                "install",
                "--prefix",
                out.to_str().unwrap(),
                "--ignore-scripts",
                "--omit",
                "dev",
            ])
            .current_dir(&src)
            .status()
            .unwrap()
            .success());
    }

    match &*platform {
        "android" => build_for_android(&src, extra_args, &tauri_root.to_path_buf()),
        "ios" => build_for_ios(&src, extra_args, &tauri_root.to_path_buf()),
        "macos" => build_for_macos(&src, extra_args),
        "linux" => build_for_linux(&src, extra_args),
        "windows" => build_for_windows(&src, extra_args),
        os => panic!("Unsupported target platform: {os}"),
    };

    generate_bindings(&src, &out);

    tauri_plugin::Builder::new(COMMANDS).build();
}

fn build_for_android<P: AsRef<Path>>(src: &P, extra_args: Vec<&str>, tauri_root: &P) {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build = out.join("build");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match &*arch {
        "aarch64" => "arm64",
        "x86" => "ia32",
        "x86_64" => "x64",
        arch => arch,
    };
    let abi = match &*arch {
        "arm" => "armeabi-v7a",
        "arm64" => "arm64-v8a",
        "ia32" => "x86",
        "x64" => "x86_64",
        abi => abi,
    };
    let dest = tauri_root
        .as_ref()
        .join(format!("gen/android/app/src/main/jniLibs/{abi}"));
    let ndk = PathBuf::from(env::var("ANDROID_NDK").unwrap());
    let host = match env::consts::OS {
        "macos" => "darwin-x86_64".to_string(),
        os => format!("{os}-x86_64"),
    };
    let triple = match arch {
        "arm" => "arm-linux-androideabi",
        "arm64" => "aarch64-linux-android",
        "ia32" => "i686-linux-android",
        "x64" => "x86_64-linux-android",
        abi => panic!("Shouldn't happen! Android ABI: {abi}"),
    };
    let libcpp = ndk.join(format!(
        "toolchains/llvm/prebuilt/{host}/sysroot/usr/lib/{triple}/libc++_shared.so"
    ));
    let mut args = vec![
        "--yes",
        MAKE,
        "generate",
        "--source",
        src.to_str().unwrap(),
        "--build",
        build.to_str().unwrap(),
        "--platform",
        "android",
        "--arch",
        arch,
        "--define",
        "ANDROID_STL=c++_shared",
    ];

    args.extend(extra_args);

    if env::var("DEBUG").unwrap() == "true" {
        args.push("--debug");
    }

    assert!(
        Command::new(RUNNER).args(args).status().unwrap().success(),
        "Configure failed"
    );
    assert!(
        Command::new(RUNNER)
            .args(["--yes", MAKE, "build", "--build", build.to_str().unwrap()])
            .status()
            .unwrap()
            .success(),
        "Build failed"
    );
    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                MAKE,
                "install",
                "--build",
                build.to_str().unwrap(),
                "--prefix",
                dest.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success(),
        "Install failed"
    );

    fs::copy(&libcpp, dest.join(&libcpp.file_name().unwrap())).unwrap();

    println!(
        "cargo::rustc-link-search=native={}",
        dest.join(arch).display()
    );
    println!("cargo::metadata=RESOURCE_DIR={}", dest.display());
    println!("cargo::rustc-link-lib=bare-kit");
}

fn build_for_ios<P: AsRef<Path>>(src: &P, extra_args: Vec<&str>, tauri_root: &P) {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build = out.join("build");
    let arch = &*env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match arch {
        "aarch64" => "arm64",
        arch => arch,
    };
    let profile = env::var("PROFILE").unwrap();
    let dest = tauri_root
        .as_ref()
        .join(format!("gen/apple/Externals/{arch}/{profile}"));
    let mut args = vec![
        "--yes",
        MAKE,
        "generate",
        "--source",
        src.to_str().unwrap(),
        "--build",
        build.to_str().unwrap(),
        "--platform",
        "ios",
        "--arch",
        arch,
    ];

    args.extend(extra_args);

    if env::var("CARGO_CFG_TARGET_ENV").unwrap() == "sim" {
        args.push("--simulator");
    }

    if env::var("DEBUG").unwrap() == "true" {
        args.push("--debug");
    }

    assert!(
        Command::new(RUNNER).args(args).status().unwrap().success(),
        "Configure failed"
    );
    assert!(
        Command::new(RUNNER)
            .args(["--yes", MAKE, "build", "--build", build.to_str().unwrap()])
            .status()
            .unwrap()
            .success(),
        "Build failed"
    );
    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                MAKE,
                "install",
                "--build",
                build.to_str().unwrap(),
                "--prefix",
                dest.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success(),
        "Install failed"
    );

    println!("cargo::metadata=RESOURCE_DIR={}", dest.display());
    println!("cargo::rustc-link-search=framework={}", dest.display());
    println!("cargo::rustc-link-lib=framework=BareKit");
}

fn build_for_macos<P: AsRef<Path>>(src: &P, extra_args: Vec<&str>) {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build = out.join("build");
    let dest = out.join("bare-kit");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match &*arch {
        "aarch64" => "arm64",
        "x86" => "ia32",
        "x86_64" => "x64",
        arch => arch,
    };
    let mut args = vec![
        "--yes",
        MAKE,
        "generate",
        "--source",
        src.to_str().unwrap(),
        "--build",
        build.to_str().unwrap(),
        "--platform",
        "darwin",
        "--arch",
        arch,
    ];

    args.extend(extra_args);

    if env::var("DEBUG").unwrap() == "true" {
        args.push("--debug");
    }

    assert!(
        Command::new(RUNNER).args(args).status().unwrap().success(),
        "Configure failed"
    );
    assert!(
        Command::new(RUNNER)
            .args(["--yes", MAKE, "build", "--build", build.to_str().unwrap()])
            .status()
            .unwrap()
            .success(),
        "Build failed"
    );
    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                MAKE,
                "install",
                "--build",
                build.to_str().unwrap(),
                "--prefix",
                dest.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success(),
        "Install failed"
    );

    println!("cargo::metadata=RESOURCE_DIR={}", dest.display());
}

fn build_for_linux<P: AsRef<Path>>(src: &P, extra_args: Vec<&str>) {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build = out.join("build");
    let dest = out.join("bare-kit");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match &*arch {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        arch => arch,
    };

    let mut args = vec![
        "--yes",
        MAKE,
        "generate",
        "--source",
        src.to_str().unwrap(),
        "--build",
        build.to_str().unwrap(),
        "--platform",
        "linux",
        "--arch",
        arch,
    ];

    args.extend(extra_args);

    if env::var("DEBUG").unwrap() == "true" {
        args.push("--debug");
    }

    assert!(
        Command::new(RUNNER).args(args).status().unwrap().success(),
        "Configure failed",
    );
    assert!(
        Command::new(RUNNER)
            .args(["--yes", MAKE, "build", "--build", build.to_str().unwrap()])
            .status()
            .unwrap()
            .success(),
        "Build failed"
    );
    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                MAKE,
                "install",
                "--build",
                build.to_str().unwrap(),
                "--prefix",
                dest.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success(),
        "Install failed"
    );

    println!("cargo::metadata=RESOURCE_DIR={}", dest.display());
    println!("cargo::rustc-link-arg=-Wl,-rpath=$ORIGIN");
    println!("cargo::rustc-link-search=native={}", dest.display());
    println!("cargo::rustc-link-lib=bare-kit");
}

fn build_for_windows<P: AsRef<Path>>(src: &P, extra_args: Vec<&str>) {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match &*arch {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        arch => arch,
    };
    let build = out.join("build");
    let dest = out.join("bare-kit");

    let mut args = vec![
        "--yes",
        MAKE,
        "generate",
        "--source",
        src.to_str().unwrap(),
        "--build",
        build.to_str().unwrap(),
        "--platform",
        "win32",
        "--arch",
        arch,
    ];

    args.extend(extra_args);

    if env::var("DEBUG").unwrap() == "true" {
        args.push("--debug");
    }

    assert!(
        Command::new(RUNNER).args(args).status().unwrap().success(),
        "Configure failed"
    );
    assert!(
        Command::new(RUNNER)
            .args(["--yes", MAKE, "build", "--build", build.to_str().unwrap()])
            .status()
            .unwrap()
            .success(),
        "Build failed"
    );
    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                MAKE,
                "install",
                "--build",
                build.to_str().unwrap(),
                "--prefix",
                dest.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success(),
        "Install failed"
    );

    println!("cargo::metadata=RESOURCE_DIR={}", dest.display());
    println!("cargo::rustc-link-search=native={}", dest.display());
    println!("cargo::rustc-link-lib=dylib=bare-kit");
}

fn generate_bindings<S: AsRef<Path>, O: AsRef<Path>>(src: &S, out: &O) {
    let src = src.as_ref();
    let out = out.as_ref();
    let header = src.join("include/bare-kit.h");

    let bindings = bindgen::Builder::default()
        .header(&*header.to_string_lossy())
        .allowlist_file(".*bare-kit\\.h")
        .allowlist_file(".*android\\.h")
        .allowlist_file(".*stdbool\\.h")
        .allowlist_file(".*stddef\\.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap();
    bindings.write_to_file(out.join("bare-kit.rs")).unwrap();
}
