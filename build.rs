use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const MAKE: &str = "bare-make@latest";
#[cfg(unix)]
const RUNNER: &str = "npx";
#[cfg(windows)]
const RUNNER: &str = "npx.cmd";

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

    let config_str = String::from_utf8(
        Command::new(env::var("CARGO").unwrap())
            .current_dir(&out)
            .arg("locate-project")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let meta_json: META = serde_json::from_str(&config_str).unwrap();
    let project_root = PathBuf::from(meta_json.root);
    let project_root = if src == project_root.parent().unwrap() {
        // Development
        &project_root.with_file_name("example/src-tauri")
    } else {
        // Installed as a dependency
        project_root.parent().unwrap()
    };
    let project = &*format!(
        "PARENT_PROJECT_PATH={}",
        project_root.parent().unwrap().display()
    );

    match &*platform {
        "android" => build_for_android(&src, project, &project_root.to_path_buf()),
        "ios" => build_for_ios(&src, project, &project_root.to_path_buf()),
        "macos" => build_for_macos(&src, project),
        "linux" => build_for_linux(&src, project),
        "windows" => build_for_windows(&src, project),
        os => panic!("Unsupported target platform: {os}"),
    };

    generate_bindings(&src, &out);

    tauri_plugin::Builder::new(COMMANDS).build();
}

fn build_for_android<P: AsRef<Path>>(src: &P, project: &str, project_root: &P) {
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
    let dest = project_root
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
        "--define",
        project,
    ];

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

fn build_for_ios<P: AsRef<Path>>(src: &P, project: &str, project_root: &P) {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build = out.join("build");
    let arch = &*env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match arch {
        "aarch64" => "arm64",
        arch => arch,
    };
    let profile = env::var("PROFILE").unwrap();
    let dest = project_root
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
        "--define",
        project,
    ];

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

fn build_for_macos<P: AsRef<Path>>(src: &P, project: &str) {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build = out.join("build");
    let scratch = out.join("scratch");
    let dest = out.join("bare-kit");
    let archs = vec!["arm64", "x64"];
    let framework = dest.join("BareKit.framework");
    let framework_bin = framework.join("Versions/A");
    let framework_head = framework_bin.join("Headers");
    let framework_res = framework_bin.join("Resources");

    fs::remove_dir_all(&framework)
        .or_else(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(err),
        })
        .unwrap();

    for arch in &archs {
        let target = format!("darwin-{arch}");
        let build = build.join(&target);
        let scratch = scratch.join(&target);
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
            "--define",
            project,
        ];

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
                    scratch.to_str().unwrap(),
                ])
                .status()
                .unwrap()
                .success(),
            "Install failed"
        );
    }

    fs::create_dir_all(&framework_bin).unwrap();
    fs::create_dir_all(&framework_head).unwrap();
    fs::create_dir_all(&framework_res).unwrap();

    let header_a = scratch
        .join(format!("darwin-{}", &archs[0]))
        .join("BareKit.framework/Versions/A/Headers/BareKit.h");
    let header_b = scratch
        .join(format!("darwin-{}", &archs[1]))
        .join("BareKit.framework/Versions/A/Headers/BareKit.h");
    assert_eq!(fs::read(&header_a).unwrap(), fs::read(&header_b).unwrap());
    fs::copy(&header_a, &framework_head.join("BareKit.h")).unwrap();

    let plist_a = scratch
        .join(format!("darwin-{}", &archs[0]))
        .join("BareKit.framework/Versions/A/Resources/Info.plist");
    let plist_b = scratch
        .join(format!("darwin-{}", &archs[1]))
        .join("BareKit.framework/Versions/A/Resources/Info.plist");
    assert_eq!(fs::read(&plist_a).unwrap(), fs::read(&plist_b).unwrap());
    fs::copy(&plist_a, &framework_res.join("Info.plist")).unwrap();

    let bin_a = scratch
        .join(format!("darwin-{}", &archs[0]))
        .join("BareKit.framework/Versions/A/BareKit");
    let bin_b = scratch
        .join(format!("darwin-{}", &archs[1]))
        .join("BareKit.framework/Versions/A/BareKit");
    assert!(Command::new("lipo")
        .args([
            "-create",
            bin_a.to_str().unwrap(),
            bin_b.to_str().unwrap(),
            "-output",
            framework_bin.join("BareKit").to_str().unwrap()
        ])
        .status()
        .unwrap()
        .success());

    #[cfg(unix)]
    os::unix::fs::symlink(
        &framework_head.strip_prefix(&framework).unwrap(),
        &framework.join("Headers"),
    )
    .unwrap();
    #[cfg(unix)]
    os::unix::fs::symlink(
        &framework_res.strip_prefix(&framework).unwrap(),
        &framework.join("Resources"),
    )
    .unwrap();
    #[cfg(unix)]
    os::unix::fs::symlink("A", &framework_bin.join("../Current")).unwrap();
    #[cfg(unix)]
    os::unix::fs::symlink(
        &framework_bin
            .join("BareKit")
            .strip_prefix(&framework)
            .unwrap(),
        &framework.join("BareKit"),
    )
    .unwrap();

    println!("cargo::metadata=RESOURCE_DIR={}", dest.display());
}

fn build_for_linux<P: AsRef<Path>>(src: &P, project: &str) {
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
        "--define",
        project,
    ];

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

fn build_for_windows<P: AsRef<Path>>(src: &P, project: &str) {
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
        "--define",
        project,
    ];

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
