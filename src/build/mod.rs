use std::{
    env, fs, os,
    path::{Path, PathBuf},
    process::Command,
};

pub fn autolink() {
    let cargo_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let bin_dir = PathBuf::from(env::var("DEP_TAURI_PLUGIN_BARE_KIT_INSTALL_DIR").unwrap());
    let lib_dir = link_addons(&cargo_dir);

    match &*platform {
        "ios" | "macos" => println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/Frameworks"),
        "linux" => println!("cargo::rustc-link-arg=-Wl,-rpath=$ORIGIN/lib"),
        _ => (),
    }

    copy_dir_all(&bin_dir, &lib_dir);

    match &*platform {
        "android" | "linux" => {
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
        }
        "ios" | "macos" => {
            println!("cargo::rustc-link-search=framework={}", lib_dir.display());

            for framework in fs::read_dir(&lib_dir).unwrap().filter_map(|f| f.ok()) {
                let libpath = framework.path();
                let libname = libpath.file_stem().unwrap().to_str().unwrap();

                println!("cargo::rustc-link-lib=framework={}", libname);
            }
        }
        _ => (),
    }

    if platform == "linux" || platform == "macos" || platform == "windows" {
        let resources = lib_dir.strip_prefix(cargo_dir).unwrap();
        let dest = if platform == "macos" {
            "Frameworks"
        } else {
            "lib"
        };

        env::set_var(
            "TAURI_CONFIG",
            format!(
                "{{ \"bundle\": {{ \"resources\": {{ \"{}\": \"{}\" }} }} }}",
                resources.display(),
                dest
            ),
        );
    }
}

fn link_addons<P: AsRef<Path>>(cargo_dir: &P) -> PathBuf {
    let cargo_dir = cargo_dir.as_ref();
    let rust_platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let rust_arch = &*env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let android_abi = match rust_arch {
        "arm" => "armeabi-v7a",
        "aarch64" => "arm64-v8a",
        rest => rest,
    };
    let node_dir = node_root(&cargo_dir);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tmp_dir = out_dir.join("addons");
    let dest_dir = match &*rust_platform {
        "android" => cargo_dir
            .join("gen/android/app/src/main/jniLibs")
            .join(android_abi),
        "ios" => cargo_dir.join("gen/apple/Frameworks"),
        "macos" => out_dir.join("Frameworks"),
        "linux" => out_dir.join("lib"),
        _ => todo!("More platforms"),
    };
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
    let simulator = env::var("CARGO_CFG_TARGET_ABI").unwrap() == "sim";
    let target = format!("{platform}-{arch}");
    let target = target + if simulator { "-simulator" } else { "" };
    let needs = match &*rust_platform {
        "android" | "linux" => "libbare-kit.so",
        "ios" | "macos" => "BareKit.framework",
        "windows" => "bare-kit.dll",
        _ => todo!("Proper error"),
    };
    let runner = "npx"; // TODO: account for windows extensions

    assert!(Command::new(runner)
        .current_dir(&node_dir)
        .args([
            "bare-link",
            "--target",
            &*target,
            "--needs",
            needs,
            "--out",
            tmp_dir.to_str().unwrap(),
        ])
        .status()
        .unwrap()
        .success());

    fs::create_dir_all(&dest_dir).unwrap();

    if tmp_dir.exists() {
        match &*rust_platform {
            "android" => {
                let bin_dir = tmp_dir.join(android_abi);

                for addon in fs::read_dir(&bin_dir).unwrap().filter_map(|a| a.ok()) {
                    fs::copy(addon.path(), dest_dir.join(addon.file_name())).unwrap();
                }
            }
            "ios" | "macos" => {
                for addon in fs::read_dir(&tmp_dir).unwrap().filter_map(|a| a.ok()) {
                    let filepath = match &*rust_platform {
                        "ios" if !simulator => addon.path().join(format!("ios-{arch}")),
                        "ios" if simulator => addon.path().join(format!("ios-{arch}-simulator")),
                        "macos" => addon.path().join(format!("macos-{arch}")),
                        _ => todo!("Proper error"),
                    };

                    copy_dir_all(&filepath, &dest_dir);
                }
            }
            "linux" => {
                let bin_dir = tmp_dir.join("lib");

                for addon in fs::read_dir(&bin_dir).unwrap().filter_map(|a| a.ok()) {
                    fs::copy(addon.path(), dest_dir.join(addon.file_name())).unwrap();
                }
            }
            _ => todo!("More platforms"),
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
