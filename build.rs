use std::{
    env, fs, io,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

const VERSION: &str = "2.4.3";

const COMMANDS: &[&str] = &[
    "bare_optimize_for_memory",
    "bare_new_worklet",
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
    let dest = out.join("bare-kit");
    let platform = env::var("CARGO_CFG_TARGET_OS").unwrap();

    fs::create_dir_all(&dest).unwrap();

    match &*platform {
        "macos" => {
            let prefix = "darwin/BareKit.xcframework/macos-arm64_x86_64/";

            extract_prebuilds(prefix, &dest);

            if cfg!(feature = "tests") {
                println!("cargo::rustc-link-arg=-Wl,-rpath,{}", dest.display());
                println!("cargo::rustc-link-search=framework={}", dest.display());
                println!("cargo::rustc-link-lib=framework=BareKit");
            }
        }
        "ios" => {
            let simulator = env::var("CARGO_CFG_TARGET_ENV").unwrap() == "sim";
            let prefix = if simulator {
                "ios/BareKit.xcframework/ios-arm64_x86_64-simulator/"
            } else {
                "ios/BareKit.xcframework/ios-arm64/"
            };

            extract_prebuilds(prefix, &dest);
        }
        _ => panic!("Unsupported platform"),
    }

    println!("cargo::metadata=RESOURCE_DIR={}", dest.display());

    generate_bindings(&src, &out);

    tauri_plugin::Builder::new(COMMANDS).build();
}

fn extract_prebuilds<P: AsRef<Path>>(prefix: &str, dest: P) {
    let dest = dest.as_ref();
    let mut archive = get_prebuilds();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();

        let Some(name) = entry.enclosed_name() else {
            continue;
        };

        if !name.starts_with(&prefix) {
            continue;
        }

        let relative = &name.to_string_lossy()[prefix.len()..];

        if relative.is_empty() {
            continue;
        }

        let output = dest.join(relative);

        if entry.is_dir() {
            fs::create_dir_all(&output).unwrap();
        } else {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent).unwrap();
            }

            io::copy(&mut entry, &mut fs::File::create(&output).unwrap()).unwrap();
        }
    }
}

fn get_prebuilds() -> ZipArchive<fs::File> {
    let uri = format!(
        "https://github.com/holepunchto/bare-kit/releases/download/v{VERSION}/prebuilds.zip"
    );
    let output = env::temp_dir().join(format!("tauri-plugin-bare-kit/{VERSION}.zip"));

    if fs::exists(&output).unwrap() {
        ZipArchive::new(fs::File::open(output).unwrap()).unwrap()
    } else {
        fs::create_dir_all(output.parent().unwrap()).unwrap();

        let mut response = reqwest::blocking::get(uri)
            .unwrap()
            .error_for_status()
            .unwrap();
        let mut archive = fs::File::create(output).unwrap();

        io::copy(&mut response, &mut archive).unwrap();
        ZipArchive::new(archive).unwrap()
    }
}

fn build_for_android<P: AsRef<Path>>(tauri_root: &P) {
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

    println!(
        "cargo::rustc-link-search=native={}",
        dest.join(arch).display()
    );
    println!("cargo::rustc-link-lib=bare-kit");
}

fn build_for_ios<P: AsRef<Path>>(tauri_root: &P) {
    let arch = &*env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match arch {
        "aarch64" => "arm64",
        arch => arch,
    };
    let profile = env::var("PROFILE").unwrap();
    let dest = tauri_root
        .as_ref()
        .join(format!("gen/apple/Externals/{arch}/{profile}"));

    println!("cargo::rustc-link-search=framework={}", dest.display());
    println!("cargo::rustc-link-lib=framework=BareKit");
}

fn build_for_linux<P: AsRef<Path>>() {
    if cfg!(feature = "tests") {
        println!("cargo::rustc-link-arg=-Wl,-rpath=//");
    } else {
        println!("cargo::rustc-link-arg=-Wl,-rpath=$ORIGIN");
    }

    println!("cargo::rustc-link-search=native=//");
    println!("cargo::rustc-link-lib=bare-kit");
}

fn build_for_windows<P: AsRef<Path>>(src: &P) {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap();
    let dest = out.join("bare-kit");

    println!("cargo::rustc-link-search=native={}", dest.display());
    println!("cargo::rustc-link-lib=dylib=bare-kit");

    if cfg!(feature = "tests") {
        fs::copy(
            dest.join("bare-kit.dll"),
            src.join(format!("target/{profile}/bare-kit.dll")),
        )
        .unwrap();
    }
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
