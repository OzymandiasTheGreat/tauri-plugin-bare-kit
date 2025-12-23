#[cfg(feature = "runtime")]
use std::{
    env, fs, os,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(feature = "runtime")]
const MAKE: &str = "bare-make@1.6.3";
#[cfg(feature = "runtime")]
const PLUGIN: &str = "tauri-plugin-bare-kit";
#[cfg(all(feature = "runtime", unix))]
const RUNNER: &str = "npx";
#[cfg(all(feature = "runtime", windows))]
const RUNNER: &str = "npx.cmd";

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
    let platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let platfrom = match &*platform {
        "macos" => "darwin",
        "windows" => "win32",
        os => os,
    };
    let dest = match platfrom {
        "android" => build_for_android(&src),
        "darwin" => build_for_darwin(&src),
        "ios" => build_for_ios(&src),
        "linux" => build_for_linux(&src),
        "win32" => build_for_windows(&src),
        os => panic!("Unsupported target platform: {os}"),
    };
    println!("cargo::metadata=CURRENT_DIR={}", dest.display(),);

    generate_bindings(&src, &out);

    tauri_plugin::Builder::new(COMMANDS).build();
}

#[cfg(feature = "runtime")]
fn build_for_android<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let build = temp.join("build");
    let scratch = temp.join("scratch");
    let dest = temp.join("lib").join("android");
    let archs = vec![
        ("arm", "armeabi-v7a"),
        ("arm64", "arm64-v8a"),
        ("ia32", "x86"),
        ("x64", "x86_64"),
    ];
    let libname = "libbare-kit.so";
    let libcpp = "libc++_shared.so";
    let ndk = PathBuf::from(env::var("ANDROID_NDK").unwrap());
    let host = match env::consts::OS {
        "macos" => "darwin-x86_64".to_string(),
        os => format!("{os}-x86_64"),
    };

    for (_, abi) in &archs {
        fs::remove_file(dest.join(abi).join(libname))
            .or_else(|err| match err.kind() {
                std::io::ErrorKind::NotFound => Ok(()),
                _ => Err(err),
            })
            .unwrap();
    }

    for (arch, abi) in archs {
        let target = format!("android-{arch}");
        let build = build.join(&target);
        let scratch = scratch.join(&target);
        let dest = dest.join(abi);
        let triple = match abi {
            "armeabi-v7a" => "arm-linux-androideabi",
            "arm64-v8a" => "aarch64-linux-android",
            "x86" => "i686-linux-android",
            "x86_64" => "x86_64-linux-android",
            abi => panic!("Shouldn't happen! Android ABI: {abi}"),
        };
        let stl = ndk
            .join(format!(
                "toolchains/llvm/prebuilt/{host}/sysroot/usr/lib/{triple}"
            ))
            .join(libcpp);
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
                    scratch.to_str().unwrap()
                ])
                .status()
                .unwrap()
                .success(),
            "Install failed"
        );

        fs::create_dir_all(&dest).unwrap();
        fs::copy(&scratch.join("lib").join(libname), &dest.join(libname)).unwrap();
        fs::copy(&stl, &dest.join(libcpp)).unwrap();
    }

    dest
}

#[cfg(feature = "runtime")]
fn build_for_darwin<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let build = temp.join("build");
    let scratch = temp.join("scratch");
    let dest = temp.join("Frameworks").join("darwin");
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
        .join("Frameworks/BareKit.framework/Versions/A/Headers/BareKit.h");
    let header_b = scratch
        .join(format!("darwin-{}", &archs[1]))
        .join("Frameworks/BareKit.framework/Versions/A/Headers/BareKit.h");
    assert_eq!(fs::read(&header_a).unwrap(), fs::read(&header_b).unwrap());
    fs::copy(&header_a, &framework_head.join("BareKit.h")).unwrap();

    let plist_a = scratch
        .join(format!("darwin-{}", &archs[0]))
        .join("Frameworks/BareKit.framework/Versions/A/Resources/Info.plist");
    let plist_b = scratch
        .join(format!("darwin-{}", &archs[1]))
        .join("Frameworks/BareKit.framework/Versions/A/Resources/Info.plist");
    assert_eq!(fs::read(&plist_a).unwrap(), fs::read(&plist_b).unwrap());
    fs::copy(&plist_a, &framework_res.join("Info.plist")).unwrap();

    let bin_a = scratch
        .join(format!("darwin-{}", &archs[0]))
        .join("Frameworks/BareKit.framework/Versions/A/BareKit");
    let bin_b = scratch
        .join(format!("darwin-{}", &archs[1]))
        .join("Frameworks/BareKit.framework/Versions/A/BareKit");
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

    dest
}

#[cfg(feature = "runtime")]
fn build_for_ios<P: AsRef<Path>>(src: &P) -> PathBuf {
    let _src = src.as_ref();
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let dest = temp.join("Frameworks").join("ios");

    dest
}

#[cfg(feature = "runtime")]
fn build_for_linux<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let arch = match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        arch => panic!("Unsupported target architecture: {arch}"),
    };
    let target = format!("linux-{arch}");
    let build = temp.join("build").join(&target);
    let scratch = temp.join("scratch").join(&target);
    let dest = temp.join("lib").join(&target);

    fs::remove_file(dest.join("libbare-kit.so"))
        .or_else(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(err),
        })
        .unwrap();

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
                scratch.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success(),
        "Install failed"
    );

    fs::create_dir_all(&dest.parent().unwrap()).unwrap();
    #[cfg(unix)]
    os::unix::fs::symlink(&scratch.join("lib"), &dest)
        .or_else(|err| {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(err)
            }
        })
        .unwrap();
    #[cfg(windows)]
    os::windows::fs::symlink_dir(&scratch.join("lib"), &dest)
        .or_else(|err| {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(err)
            }
        })
        .unwrap();

    dest
}

#[cfg(feature = "runtime")]
fn build_for_windows<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let arch = match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        arch => panic!("Unsupported target architecture: {arch}"),
    };
    let target = format!("win32-{arch}");
    let build = temp.join("build").join(&target);
    let scratch = temp.join("scratch").join(&target);
    let bin = temp.join("bin").join(&target);
    let lib = temp.join("lib").join(&target);

    fs::remove_file(bin.join("bare-kit.dll"))
        .or_else(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(err),
        })
        .unwrap();
    fs::remove_file(lib.join("bare-kit.lib"))
        .or_else(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(err),
        })
        .unwrap();

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
                scratch.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success(),
        "Install failed"
    );

    fs::create_dir_all(&bin.parent().unwrap()).unwrap();
    #[cfg(windows)]
    os::windows::fs::symlink_dir(&scratch.join("bin"), &bin)
        .or_else(|err| {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(err)
            }
        })
        .unwrap();
    fs::create_dir_all(&lib.parent().unwrap()).unwrap();
    #[cfg(windows)]
    os::windows::fs::symlink_dir(&scratch.join("lib"), &lib)
        .or_else(|err| {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(err)
            }
        })
        .unwrap();

    bin
}

#[cfg(feature = "runtime")]
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
