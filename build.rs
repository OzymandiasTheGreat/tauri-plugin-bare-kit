#[cfg(feature = "runtime")]
use std::{
    env, fs,
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

#[cfg(feature = "runtime")]
const INSTALL_DIR: &str = "install";

#[cfg(feature = "runtime")]
type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "runtime")]
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    BareKit(String),
    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Unicode(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Bindgen(#[from] bindgen::BindgenError),
}

#[cfg(feature = "runtime")]
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::BareKit(value.into())
    }
}

#[cfg(not(feature = "runtime"))]
fn main() {}

#[cfg(feature = "runtime")]
fn main() -> Result<()> {
    let source_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let platform = env::var("CARGO_CFG_TARGET_OS")?;
    let install = build(&source_dir, &out_dir)?.join(if platform == "ios" || platform == "macos" {
        "Frameworks"
    } else {
        "lib"
    });

    match &*platform {
        "android" => {
            let ndk = env::var("ANDROID_NDK_HOME")?;
            let host = match env::consts::OS {
                "macos" => "darwin-x86_64",
                "linux" => "linux-x86_64",
                "windows" => "windows-x86_64",
                _ => return Err("Unsupported host operating system".into()),
            };
            let rust_target = env::var("TARGET")?;
            let target = match &*rust_target {
                "armv7-linux-androideabi" => "arm-linux-androideabi",
                rest => rest,
            };
            let stl = format!(
                "{ndk}/toolchains/llvm/prebuilt/{host}/sysroot/usr/lib/{target}/libc++_shared.so"
            );
            fs::copy(stl, install.join("libc++_shared.so"))?;
        }
        _ => (),
    };

    generate_bindings(&source_dir, &out_dir)?;

    println!("cargo::metadata=INSTALL_DIR={}", install.display());

    tauri_plugin::Builder::new(COMMANDS).build();

    Ok(())
}

#[cfg(feature = "runtime")]
fn build<P: AsRef<Path>>(source_dir: &P, out_dir: &P) -> Result<PathBuf> {
    let source = source_dir.as_ref();
    let out = out_dir.as_ref();
    let build = out.join("build");
    let install = out.join(INSTALL_DIR);

    let platform = env::var("CARGO_CFG_TARGET_OS")?;
    let platform = match &*platform {
        "macos" => "darwin",
        "windows" => "win32",
        rest => rest,
    };
    let arch = match &*env::var("CARGO_CFG_TARGET_ARCH")? {
        "arm" => "arm",
        "aarch64" => "arm64",
        "x86" => "ia32",
        "x86_64" => "x64",
        _ => return Err("Unsupported target architecture".into()),
    };
    // TODO: windows needs extension
    let runner = "npx";
    let make = "bare-make";
    let mut args = vec![
        make,
        "generate",
        "--source",
        source.to_str().unwrap(),
        "--build",
        build.to_str().unwrap(),
        "--platform",
        platform,
        "--arch",
        arch,
    ];

    #[cfg(target_vendor = "apple")]
    if env::var("CARGO_CFG_TARGET_ABI")? == "sim" {
        args.push("--simulator");
    }

    if env::var("DEBUG")? == "true" {
        args.push("--debug");
    }

    if platform == "android" {
        args.append(&mut vec!["-D", "ANDROID_STL=c++_shared"]);
    }

    assert!(Command::new(runner).args(args).status()?.success());
    assert!(Command::new(runner)
        .args([
            make,
            "build",
            "--build",
            build.to_str().unwrap(),
            "--target",
            "bare_kit",
        ])
        .status()?
        .success());
    assert!(Command::new(runner)
        .args([
            make,
            "install",
            "--build",
            build.to_str().unwrap(),
            "--prefix",
            install.to_str().unwrap(),
        ])
        .status()?
        .success());

    Ok(install)
}

#[cfg(feature = "runtime")]
fn generate_bindings<P: AsRef<Path>>(source_dir: &P, out_dir: &P) -> Result<()> {
    let source = source_dir.as_ref();
    let out = out_dir.as_ref();
    let target = env::var("CARGO_CFG_TARGET_OS")?;
    let header = source.join("include/bare-kit.h");

    let sysroot = match &*target {
        "android" => {
            let ndk = env::var("ANDROID_NDK_HOME")?;
            let host = match env::consts::OS {
                "linux" => "linux-x86_64",
                "macos" => "darwin-x86_64",
                "windows" => "windows-x86_64",
                _ => return Err("Unrecognized host".into()),
            };
            format!("{ndk}/toolchains/llvm/prebuilt/{host}/sysroot")
        }
        "ios" | "macos" => {
            let simulator = env::var("CARGO_CFG_TARGET_ABI")? == "sim";
            let sdk = match &*target {
                "ios" if !simulator => "iphoneos",
                "ios" if simulator => "iphonesimulator",
                "macos" => "macosx",
                _ => return Err("Unexpected target".into()),
            };
            let output = Command::new("xcrun")
                .args(["--show-sdk-path", "--sdk", sdk])
                .output()?;
            let output = String::from_utf8(output.stdout)?;
            output.trim().to_owned()
        }
        "linux" => "".to_string(),
        _ => return Err("Unexpected target".into()),
    };
    let args = match &*target {
        "android" => vec![
            "--sysroot".to_string(),
            sysroot,
            format!(
                "-I{}",
                out.join(format!("{INSTALL_DIR}/include/include")).display()
            ),
        ],
        "ios" | "macos" => vec![
            "-isysroot".to_string(),
            sysroot,
            format!(
                "-I{}",
                out.join(format!("{INSTALL_DIR}/include/include")).display()
            ),
        ],
        "linux" => vec![format!(
            "-I{}",
            out.join(format!("{INSTALL_DIR}/include/include")).display()
        )],
        _ => return Err("Unexpected target".into()),
    };

    let bindings = bindgen::Builder::default()
        .header(header.to_str().unwrap())
        .clang_args(args);
    let bindings = match &*target {
        "android" => bindings
            .allowlist_file(".*android/ipc\\.h")
            .allowlist_file(".*android/suspension\\.h"),
        "ios" | "macos" => bindings
            .allowlist_file(".*apple/ipc\\.h")
            .allowlist_file(".*apple/suspension\\.h"),
        "linux" => bindings
            .allowlist_file(".*linux/ipc\\.h")
            .allowlist_file(".*linux/suspension\\.h"),
        _ => bindings,
    };
    let bindings = bindings
        .allowlist_file(".*bare-kit\\.h")
        .allowlist_file(".*bare\\.h")
        .allowlist_file(".*js\\.h")
        .allowlist_file(".*uv\\.h")
        .allowlist_file(".*stdbool\\.h")
        .allowlist_file(".*stddef\\.h");
    let bindings = match &*target {
        "android" => bindings.allowlist_file(".*android/looper\\.h"),
        "ios" | "macos" => bindings
            .allowlist_file(".*dispatch/dispatch\\.h")
            .allowlist_file(".*stdatomic\\.h"),
        "linux" => bindings
            .allowlist_file(".*pthread\\.h")
            .allowlist_file(".*sys/epoll\\.h"),
        _ => bindings,
    };
    let bindings = bindings
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?;
    bindings.write_to_file(out.join("bindings.rs"))?;

    Ok(())
}
