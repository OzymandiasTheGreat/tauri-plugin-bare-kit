use std::{
    env, fs, os,
    path::{Path, PathBuf},
    process::Command,
};

use regex::RegexBuilder;

pub fn autolink() {
    let target_platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let android_abi = match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "arm" => "armeabi-v7a",
        "aarch64" => "arm64-v8a",
        "x86" => "x86",
        "x86_64" => "x86_64",
        abi => panic!("Unsupported architecture: {abi}"),
    };

    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let plugin_install_dir =
        PathBuf::from(env::var("DEP_TAURI_PLUGIN_BARE_KIT_INSTALL_DIR").unwrap());
    let app_lib_dir = match &*target_platform {
        "android" => {
            let dir = src
                .join("gen/android/app/src/main/jniLibs")
                .join(android_abi);

            fs::create_dir_all(&dir).unwrap();
            dir.canonicalize().unwrap()
        }
        "ios" => {
            let dir = src.join("gen/apple/Frameworks");

            fs::create_dir_all(&dir).unwrap();
            dir.canonicalize().unwrap()
        }
        "macos" => {
            let dir = out.join("Frameworks");

            fs::create_dir_all(&dir).unwrap();
            dir.canonicalize().unwrap()
        }
        "linux" => {
            let dir = out.join("lib");

            fs::create_dir_all(&dir).unwrap();
            dir.canonicalize().unwrap()
        }
        "windows" => {
            let dir = out.join("bin");

            fs::create_dir_all(&dir).unwrap();
            dir.canonicalize().unwrap()
        }
        os => panic!("Unsupported platform: {os}"),
    };

    link_addons(&src, &out, &app_lib_dir);

    match &*target_platform {
        "android" => {
            println!("cargo::rustc-link-search=native={}", app_lib_dir.display());

            for lib in fs::read_dir(&app_lib_dir).unwrap().filter_map(|l| l.ok()) {
                let filepath = lib.path();
                let filename = &*filepath.file_stem().unwrap().to_string_lossy();
                let libname = filename.strip_prefix("lib").unwrap_or(filename);

                println!("cargo::rustc-link-lib={libname}");
            }

            copy_dir_all(&plugin_install_dir, &app_lib_dir);
            println!("cargo::rustc-link-lib=bare-kit");

            let cpp = "libc++_shared.so";
            let ndk = env::var("ANDROID_NDK_HOME").unwrap();
            let host = match env::consts::OS {
                "macos" => "darwin-x86_64",
                "linux" => "linux-x86_64",
                "windows" => "windows-x86_64",
                host => panic!("Unsupported host operating system: {host}"),
            };
            let rust_target = env::var("TARGET").unwrap();
            let target = match &*rust_target {
                "armv7-linux-androideabi" => "arm-linux-androideabi",
                target => target,
            };
            let stl =
                format!("{ndk}/toolchains/llvm/prebuilt/{host}/sysroot/usr/lib/{target}/{cpp}");
            fs::copy(stl, &app_lib_dir.join(cpp)).unwrap();
        }
        "ios" | "macos" => {
            println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/Frameworks");
            println!(
                "cargo::rustc-link-search=framework={}",
                app_lib_dir.display()
            );

            for framework in fs::read_dir(&app_lib_dir).unwrap().filter_map(|f| f.ok()) {
                let framework_path = framework.path();
                let framework_name = &*framework_path.file_stem().unwrap().to_string_lossy();

                println!("cargo::rustc-link-lib=framework={framework_name}");
            }

            copy_dir_all(&plugin_install_dir, &app_lib_dir);
            println!("cargo::rustc-link-lib=framework=BareKit");
        }
        "linux" => {
            println!("cargo::rustc-link-arg=-Wl,-rpath=$ORIGIN/lib");
            println!("cargo::rustc-link-search=native={}", app_lib_dir.display());

            for lib in fs::read_dir(&app_lib_dir).unwrap().filter_map(|l| l.ok()) {
                let filepath = lib.path();
                let filename = &*filepath.file_stem().unwrap().to_string_lossy();
                let libname = filename.strip_prefix("lib").unwrap_or(filename);

                println!("cargo::rustc-link-lib={libname}");
            }

            copy_dir_all(&plugin_install_dir, &app_lib_dir);
            println!("cargo::rustc-link-lib=bare-kit");
        }
        "windows" => {
            let plugin_lib_dir = plugin_install_dir.join("../lib").canonicalize().unwrap();
            let windows_lib_dir = app_lib_dir.join("../lib");

            fs::create_dir_all(&windows_lib_dir).unwrap();

            let windows_lib_dir = windows_lib_dir.canonicalize().unwrap();

            println!(
                "cargo::rustc-link-search=native={}",
                windows_lib_dir.display()
            );

            for lib in fs::read_dir(&app_lib_dir).unwrap().filter_map(|l| l.ok()) {
                let filepath = lib.path();
                let libname = &*filepath.file_stem().unwrap().to_string_lossy();

                println!("cargo::rustc-link-lib=dylib={libname}");
            }

            copy_dir_all(&plugin_lib_dir, &windows_lib_dir);
            copy_dir_all(&plugin_install_dir, &app_lib_dir);
            println!("cargo::rustc-link-lib=dylib=bare-kit");
        }
        _ => (),
    }

    if target_platform == "linux" || target_platform == "macos" || target_platform == "windows" {
        #[cfg(unix)]
        let resources = pathdiff::diff_paths(&app_lib_dir, &src).unwrap();
        #[cfg(windows)]
        let resources = pathdiff::diff_paths(
            app_lib_dir
                .to_string_lossy()
                .strip_prefix("\\\\?\\")
                .unwrap_or(&*app_lib_dir.to_string_lossy()),
            &src,
        )
        .unwrap();
        let resources = resources.to_string_lossy().escape_default().to_string();
        let bundle_dest = match &*target_platform {
            "linux" => "lib",
            "macos" => "Frameworks",
            "windows" => "",
            os => panic!("Unexpected platform: {os}"),
        };

        env::set_var(
            "TAURI_CONFIG",
            format!(
                "{{ \"bundle\": {{ \"resources\": {{ \"{}\": \"{}\" }} }} }}",
                resources, bundle_dest
            ),
        );
    }
}

fn link_addons<S: AsRef<Path>, O: AsRef<Path>, D: AsRef<Path>>(src: &S, out: &O, dest: &D) {
    let src = src.as_ref();
    let out = out.as_ref();
    let dest = dest.as_ref();
    let node_root = src.join("..").canonicalize().unwrap();
    assert!(
        node_root.join("package.json").exists(),
        "Could not locate `package.json` at {}",
        node_root.display()
    );

    let target_platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "arm" => "arm",
        "aarch64" => "arm64",
        "x86" => "ia32",
        "x86_64" => "x64",
        arch => panic!("Unsupported architecture: {arch}"),
    };
    let simulator = env::var("CARGO_CFG_TARGET_ABI").unwrap() == "sim";

    let addon_bin_dir = match &*target_platform {
        "android" | "linux" => {
            let dir = out.join("addons/lib");

            fs::create_dir_all(&dir).unwrap();
            dir.canonicalize().unwrap()
        }
        "ios" | "macos" => {
            let dir = out.join("addons/Frameworks");

            fs::create_dir_all(&dir).unwrap();
            dir.canonicalize().unwrap()
        }
        "windows" => {
            let dir = out.join("addons/bin");

            fs::create_dir_all(&dir).unwrap();
            dir.canonicalize().unwrap()
        }
        os => panic!("Unsupported platform: {os}"),
    };

    #[cfg(unix)]
    let runner = "npx";
    #[cfg(windows)]
    let runner = "npx.cmd";

    let (target, needs) = match &*target_platform {
        "android" => (format!("android={target_arch}"), "libbare-kit.so"),
        "ios" => (
            format!(
                "ios-{target_arch}{}",
                if simulator { "-simulator" } else { "" }
            ),
            "BareKit.framework",
        ),
        "macos" => (format!("darwin-{target_arch}"), "BareKit.framework"),
        "linux" => (format!("linux-{target_arch}"), "libbare-kit.so"),
        "windows" => (format!("win32-{target_arch}"), "bare-kit.dll"),
        os => panic!("Unsupported platform: {os}"),
    };
    let args = vec![
        "bare-link",
        "--target",
        &*target,
        "--needs",
        needs,
        "--out",
        addon_bin_dir.to_str().unwrap(),
    ];

    assert!(
        Command::new(runner)
            .current_dir(&node_root)
            .args(args)
            .status()
            .unwrap()
            .success(),
        "Failed to run bare-link"
    );

    if addon_bin_dir.exists() {
        match &*target_platform {
            "android" => {
                let android_abi = match target_arch {
                    "arm" => "armeabi-v7a",
                    "aarch64" => "arm64-v8a",
                    abi => abi,
                };
                let addon_bin_dir = addon_bin_dir.join(android_abi);

                for addon in fs::read_dir(&addon_bin_dir).unwrap().filter_map(|a| a.ok()) {
                    fs::copy(&addon.path(), &dest.join(addon.file_name())).unwrap();
                }
            }
            "ios" | "macos" => {
                for addon in fs::read_dir(&addon_bin_dir).unwrap().filter_map(|a| a.ok()) {
                    let framework_dir = match &*target_platform {
                        "ios" if simulator => {
                            addon.path().join(format!("ios-{target_arch}-simulator"))
                        }
                        "ios" if !simulator => addon.path().join(format!("ios-{target_arch}")),
                        "macos" => addon.path().join(format!("macos-{target_arch}")),
                        os => panic!("Unsupported platform: {os}"),
                    };

                    copy_dir_all(&framework_dir, &dest);
                }
            }
            "linux" => {
                let addon_bin_dir = addon_bin_dir.join("lib");

                for addon in fs::read_dir(&addon_bin_dir).unwrap().filter_map(|a| a.ok()) {
                    fs::copy(&addon.path(), &dest.join(addon.file_name())).unwrap();
                }
            }
            "windows" => {
                let addon_lib_dir = addon_bin_dir.join("../lib");
                let lib_dest = dest.join("../lib");

                fs::create_dir_all(&addon_lib_dir).unwrap();
                fs::create_dir_all(&lib_dest).unwrap();

                let addon_lib_dir = addon_lib_dir.canonicalize().unwrap();
                let lib_dest = lib_dest.canonicalize().unwrap();

                for addon in fs::read_dir(&addon_bin_dir).unwrap().filter_map(|a| a.ok()) {
                    let def =
                        addon_lib_dir.join(addon.path().with_extension("def").file_name().unwrap());

                    fs::write(&def, windows_dumpbin(&addon.path())).unwrap();

                    let lib = windows_lib(&def, &addon_lib_dir);

                    fs::copy(&addon.path(), &dest.join(addon.file_name())).unwrap();
                    fs::copy(&lib, &lib_dest.join(lib.file_name().unwrap())).unwrap();
                }
            }
            os => panic!("Unsupported platform: {os}"),
        }
    }
}

fn copy_dir_all<S: AsRef<Path>, D: AsRef<Path>>(src: &S, dest: &D) {
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

                #[cfg(unix)]
                os::unix::fs::symlink(fs::read_link(&entry.path()).unwrap(), &dest).unwrap();
                #[cfg(windows)]
                os::windows::fs::symlink_file(fs::read_link(&entry.path()).unwrap(), &dest)
                    .unwrap();
            }
        }
    }
}

fn windows_dumpbin<P: AsRef<Path>>(dll: &P) -> String {
    let dll = dll.as_ref();
    let pattern = RegexBuilder::new(r"^\s+?\d+?\s+?[\dA-F]+?\s[\dA-F]+?\s(?P<symbol>\w+)$")
        .multi_line(true)
        .crlf(true)
        .build()
        .unwrap();
    let output = Command::new("dumpbin.exe")
        .args(["/EXPORTS", &*dll.to_string_lossy()])
        .output()
        .unwrap();
    let dump_str = &*String::from_utf8_lossy(output.stdout.as_slice());
    let header = format!("EXPORTS\r\n");
    let symbols: Vec<&str> = pattern
        .captures_iter(dump_str)
        .map(|c| c.name("symbol").unwrap().as_str())
        .collect();

    header + &*symbols.join("\r\n")
}

fn windows_lib<D: AsRef<Path>, O: AsRef<Path>>(def: &D, out: O) -> PathBuf {
    let def = def.as_ref();
    let out = out
        .as_ref()
        .join(&*def.with_extension("lib").file_name().unwrap());
    let machine = match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "arm" => "arm",
        "aarch64" => "arm64",
        "x86" => "x86",
        "x86_64" => "x64",
        arch => panic!("Unsupported architecture: {arch}"),
    };

    assert!(Command::new("lib.exe")
        .args([
            format!("/DEF:{}", def.display()),
            format!("/OUT:{}", out.display()),
            format!("/MACHINE:{machine}"),
        ])
        .status()
        .unwrap()
        .success());

    out
}
