use std::{
    env, fs, os,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let bin_dir = PathBuf::from(env::var("DEP_TAURI_PLUGIN_BARE_KIT_INSTALL_DIR").unwrap());
    let lib_dir = link_addons();

    copy_dir_all(&bin_dir, &lib_dir);

    println!("cargo::rustc-link-search=native={}", lib_dir.display());

    for lib in fs::read_dir(&lib_dir).unwrap().filter_map(|l| l.ok()) {
        let filepath = lib.path();
        let filename = filepath.file_stem().unwrap().to_str().unwrap();
        let libname = filename.strip_prefix("lib").unwrap_or(filename);

        // Don't link libs provided by tauri/android
        if libname == "tauri_app_lib" || libname == "c++_shared" {
            continue;
        }

        println!("cargo::rustc-link-lib={}", libname);
    }

    tauri_build::build()
}

fn link_addons() -> PathBuf {
    let rust_arch = &*env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let android_abi = match rust_arch {
        "arm" => "armeabi-v7a",
        "aarch64" => "arm64-v8a",
        rest => rest,
    };
    let cargo_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let node_dir = node_root(&cargo_dir);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("addons");
    let dest_dir = cargo_dir
        .join("gen/android/app/src/main/jniLibs")
        .join(android_abi);
    let rust_platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let platform = match &*rust_platform {
        "macos" => "darwin",
        "windows" => "win32",
        rest => rest,
    };
    let arch = match rust_arch {
        "aarch64" => "arm64",
        "x86" => "ia32",
        "x86_64" => "x64",
        rest => rest,
    };
    let target = format!("{platform}-{arch}"); // TODO: account for simulator
    let runner = "npx"; // TODO: account for windows extensions

    assert!(Command::new(runner)
        .current_dir(&node_dir)
        .args([
            "bare-link",
            "--target",
            &*target,
            "--needs",
            "libbare-kit.so", // TODO: account for other platforms
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .status()
        .unwrap()
        .success());

    fs::create_dir_all(&dest_dir).unwrap();

    if out_dir.exists() {
        let bin_dir = out_dir.join(android_abi);

        for addon in fs::read_dir(&bin_dir).unwrap().filter_map(|a| a.ok()) {
            fs::copy(addon.path(), dest_dir.join(addon.file_name())).unwrap();
        }
    }

    dest_dir
}

fn node_root<P: AsRef<Path>>(cargo_dir: &P) -> PathBuf {
    const NODE_MANIFEST: &str = "package.json";
    let cargo_dir = cargo_dir.as_ref();

    if cargo_dir.join(NODE_MANIFEST).exists() {
        return cargo_dir.to_path_buf();
    }

    let mut parent_dir = cargo_dir.parent();

    while let Some(current_dir) = parent_dir {
        if current_dir.join(NODE_MANIFEST).exists() {
            return current_dir.to_path_buf();
        }

        parent_dir = current_dir.parent();
    }

    panic!("Reached file system root");
}

fn copy_dir_all<P: AsRef<Path>>(src: &P, dest: &P) {
    let src = src.as_ref();
    let dest = dest.as_ref();

    let err = fs::create_dir_all(&dest).err();

    if let Some(err) = err {
        match err.kind() {
            std::io::ErrorKind::AlreadyExists => (),
            _ => panic!("IO Error: {}", dest.display()),
        }
    }

    for entry in fs::read_dir(&src).unwrap().filter_map(|e| e.ok()) {
        let filetype = entry.file_type().unwrap();

        if filetype.is_dir() {
            copy_dir_all(&entry.path(), &dest.join(entry.file_name()));
        } else {
            let is_err = fs::copy(&entry.path(), &dest.join(entry.file_name())).is_err();

            if is_err && filetype.is_symlink() {
                let dest = dest.join(entry.file_name());
                let err = fs::remove_file(&dest).err();

                if let Some(err) = err {
                    match err.kind() {
                        std::io::ErrorKind::NotFound => (),
                        _ => panic!("IO Error: {}", dest.display()),
                    }
                }

                os::unix::fs::symlink(fs::read_link(&entry.path()).unwrap(), &dest).unwrap();
            }
        }
    }
}
