use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const COMMANDS: &[&str] = &[
    "ping",
    "bare_invalidate",
    "bare_new",
    "bare_start",
    "bare_read",
    "bare_write",
    "bare_update",
    "bare_suspend",
    "bare_resume",
    "bare_terminate",
];

const INSTALL_DIR: &str = "install";

type Result<T> = std::result::Result<T, Error>;

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

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::BareKit(value.into())
    }
}

fn main() -> Result<()> {
    let source_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let install = build(&source_dir, &out_dir)?.join("lib");
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
    let stl =
        format!("{ndk}/toolchains/llvm/prebuilt/{host}/sysroot/usr/lib/{target}/libc++_shared.so");
    fs::copy(stl, install.join("libc++_shared.so"))?;

    generate_bindings(&source_dir, &out_dir)?;

    println!("cargo::metadata=INSTALL_DIR={}", install.display());

    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();

    Ok(())
}

fn build<P: AsRef<Path>>(source_dir: &P, out_dir: &P) -> Result<PathBuf> {
    let source = source_dir.as_ref();
    let out = out_dir.as_ref();
    let build = out.join("build");
    let install = out.join(INSTALL_DIR);

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
        // TODO: ios, etc.
        "android",
        "--arch",
        arch,
    ];

    if env::var("DEBUG")? == "true" {
        args.push("--debug");
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

fn generate_bindings<P: AsRef<Path>>(source_dir: &P, out_dir: &P) -> Result<()> {
    let source = source_dir.as_ref();
    let out = out_dir.as_ref();
    let header = source.join("include/bare-kit.h");

    let ndk = env::var("ANDROID_NDK_HOME")?;
    let host = match env::consts::OS {
        "macos" => "darwin-x86_64",
        "linux" => "linux-x86_64",
        "windows" => "windows-x86_64",
        _ => return Err("Unsupported host operating system".into()),
    };
    let sysroot = format!("{ndk}/toolchains/llvm/prebuilt/{host}/sysroot");

    let args = vec![
        "--sysroot".to_owned(),
        sysroot,
        format!(
            "-I{}",
            out.join(format!("{INSTALL_DIR}/include/include")).display()
        ),
    ];

    let bindings = bindgen::Builder::default()
        .header(header.to_str().unwrap())
        .clang_args(args)
        .blocklist_file(".*stdlib\\.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?;
    bindings.write_to_file(out.join("bindings.rs"))?;

    Ok(())
}
