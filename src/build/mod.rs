use std::{
    env, fs, os,
    path::{Path, PathBuf},
    process::Command,
};

const LINK: &str = "bare-link@2.1.10";
const PACK: &str = "bare-pack@1.5.1";
const PLUGIN: &str = "tauri-plugin-bare-kit";
#[cfg(unix)]
const RUNNER: &str = "npx";
#[cfg(windows)]
const RUNNER: &str = "npx.cmd";

pub fn autolink() {
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let platform = match &*platform {
        "macos" => "darwin",
        "windows" => "win32",
        os => os,
    };
    let out = match platform {
        "android" => link_for_android(&src),
        "darwin" => link_for_darwin(&src),
        "ios" => link_for_ios(&src),
        "linux" => link_for_linux(&src),
        "win32" => link_for_windows(&src),
        os => panic!("Unsupported target platform: {os}"),
    };

    if platform == "darwin" || platform == "linux" || platform == "win32" {
        #[cfg(unix)]
        let relative = pathdiff::diff_paths(&out, &src)
            .unwrap()
            .to_string_lossy()
            .to_string();
        #[cfg(windows)]
        let relative = pathdiff::diff_paths(&out, &src)
            .unwrap()
            .to_string_lossy()
            .escape_default()
            .to_string();
        let dest = match platform {
            "darwin" => "Frameworks",
            "linux" => "lib",
            _ => "",
        };

        unsafe {
            env::set_var(
                "TAURI_CONFIG",
                format!(
                    "{{ \"bundle\": {{ \"resources\": {{ \"{}\": \"{}\" }} }} }}",
                    relative, dest,
                ),
            )
        };
    }
}

fn link_for_android<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    let out = src.join("gen/android/app/src/main/jniLibs");
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let dest = temp.join("lib").join("android");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match &*arch {
        "arm" => "armeabi-v7a",
        "aarch64" => "arm64-v8a",
        arch => arch,
    };
    let node = src.parent().unwrap();
    assert!(
        node.join("package.json").exists(),
        "Could not find package.json in {}",
        node.display()
    );
    let entry = node.join("bare/index.js");
    let builtins = node.join("bare/builtins.json");
    let bundle = node.join("bare/index.bundle.json");

    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                LINK,
                "--preset",
                "android",
                "--out",
                dest.to_str().unwrap(),
            ])
            .current_dir(&node)
            .status()
            .unwrap()
            .success(),
        "Linking failed"
    );

    println!(
        "cargo::rustc-link-search=native={}",
        out.join(arch).display()
    );

    for dir in fs::read_dir(&dest).unwrap().filter_map(|d| {
        if let Some(d) = d.ok() {
            if d.path().is_dir() {
                return Some(d);
            }
        }
        return None;
    }) {
        fs::create_dir_all(&out.join(dir.file_name())).unwrap();

        for lib in fs::read_dir(&dir.path()).unwrap().filter_map(|so| {
            if let Some(so) = so.ok() {
                if so.file_name().to_string_lossy().ends_with(".so") {
                    return Some(so);
                }
            }
            return None;
        }) {
            fs::copy(
                &lib.path(),
                &out.join(dir.file_name()).join(lib.file_name()),
            )
            .unwrap();

            if dir.file_name() == arch {
                let filepath = lib.path();
                let filename = &*filepath.file_stem().unwrap().to_string_lossy();
                let libname = filename.strip_prefix("lib").unwrap_or(filename);

                println!("cargo::rustc-link-lib={libname}");
            }
        }
    }

    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                PACK,
                "--preset",
                "android",
                "--builtins",
                builtins.to_str().unwrap(),
                "--linked",
                "--out",
                bundle.to_str().unwrap(),
                entry.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success(),
        "Bundling failed"
    );

    out
}

fn link_for_darwin<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("Frameworks");
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let dest = temp.join("Frameworks").join("darwin");
    let node = src.parent().unwrap();
    assert!(
        node.join("package.json").exists(),
        "Could not find package.json in {}",
        node.display()
    );
    let entry = node.join("bare/index.js");
    let builtins = node.join("bare/builtins.json");
    let bundle = node.join("bare/index.bundle.json");

    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                LINK,
                "--preset",
                "darwin",
                "--out",
                dest.to_str().unwrap()
            ])
            .current_dir(&node)
            .status()
            .unwrap()
            .success(),
        "Linking failed"
    );

    println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/Frameworks");
    println!("cargo::rustc-link-search=framework={}", out.display());

    for framework in fs::read_dir(&dest).unwrap().filter_map(|f| {
        if let Some(f) = f.ok() {
            if f.file_name().to_string_lossy().ends_with(".framework") {
                return Some(f);
            }
        }
        return None;
    }) {
        let framework_path = framework.path();
        let framework_name = &*framework_path.file_stem().unwrap().to_string_lossy();

        println!("cargo::rustc-link-lib=framework={framework_name}");
    }

    #[cfg(unix)]
    os::unix::fs::symlink(&dest, &out)
        .or_else(|err| {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(err)
            }
        })
        .unwrap();

    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                PACK,
                "--preset",
                "darwin",
                "--builtins",
                builtins.to_str().unwrap(),
                "--linked",
                "--out",
                bundle.to_str().unwrap(),
                entry.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success(),
        "Bundling failed"
    );

    out
}

fn link_for_ios<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    let out = src.join("gen/apple/Frameworks");
    let node = src.parent().unwrap();
    assert!(
        node.join("package.json").exists(),
        "Could not find package.json in {}",
        node.display()
    );
    let entry = node.join("bare/index.js");
    let builtins = node.join("bare/builtins.json");
    let bundle = node.join("bare/index.bundle.json");
    let arch = match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        arch => panic!("Unsupported target architecture for iOS: {arch}"),
    };
    let simulator = if env::var("CARGO_CFG_TARGET_ABI").unwrap() == "sim" {
        "-simulator"
    } else {
        ""
    };

    for xcframework in fs::read_dir(&out).unwrap().filter_map(|f| {
        if let Some(f) = f.ok() {
            if f.file_name().to_string_lossy().ends_with(".xcframework") {
                return Some(f);
            }
        }
        return None;
    }) {
        let filepath = xcframework.path();
        let framework = &*filepath.file_stem().unwrap().to_string_lossy();
        let searchpath = filepath.join(format!("ios-{arch}{simulator}"));
        let searchpath = if searchpath.exists() {
            searchpath
        } else {
            filepath.join(format!("ios-arm64_x86_64{simulator}"))
        };

        println!(
            "cargo::rustc-link-search=framework={}",
            searchpath.display()
        );
        println!("cargo::rustc-link-lib=framework={framework}");
    }

    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                PACK,
                "--preset",
                "ios",
                "--builtins",
                builtins.to_str().unwrap(),
                "--linked",
                "--out",
                bundle.to_str().unwrap(),
                entry.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success(),
        "Bundling failed"
    );

    out
}

fn link_for_linux<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    todo!("Link Linux!");
}

fn link_for_windows<P: AsRef<Path>>(src: &P) -> PathBuf {
    fn dumpbin_exe<P: AsRef<Path>>(dll: &P) -> String {
        let dll = dll.as_ref();
        let pattern =
            regex::RegexBuilder::new(r"^\s+?\d+?\s+?[\dA-F]+?\s[\dA-F]+?\s(?P<symbol>\w+)$")
                .multi_line(true)
                .crlf(true)
                .build()
                .unwrap();
        let raw_output = Command::new("dumpbin.exe")
            .args(["/EXPORTS", &*dll.to_string_lossy()])
            .output()
            .unwrap();
        let str_output = &*String::from_utf8_lossy(&raw_output.stdout);
        let header = "EXPORTS\r\n".to_string();
        let symbols: Vec<String> = pattern
            .captures_iter(str_output)
            .map(|c| "\t".to_string() + c.name("symbol").unwrap().as_str())
            .collect();
        header + &*symbols.join("\r\n")
    }

    fn lib_exe<I: AsRef<Path>, O: AsRef<Path>>(machine: &str, def: &I, out: &O) {
        let def = def.as_ref();
        let out = out.as_ref();
        assert!(Command::new("lib.exe")
            .args([
                format!("/DEF:{}", def.display()),
                format!("/OUT:{}", out.display()),
                format!("/MACHINE:{machine}"),
            ])
            .status()
            .unwrap()
            .success());
    }

    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bin");
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let arch = match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        arch => panic!("Unsupported target architecture: {arch}"),
    };
    let target = format!("win32-{arch}");
    let bin = temp.join("bin").join(&target);
    let lib = temp.join("lib").join(&target);
    let node = src.parent().unwrap();
    assert!(
        node.join("package.json").exists(),
        "Could not find package.json in {}",
        node.display()
    );
    let entry = node.join("bare/index.js");
    let builtins = node.join("bare/builtins.json");
    let bundle = node.join("bare/index.bundle.json");

    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                LINK,
                "--target",
                &*target,
                "--out",
                bin.to_str().unwrap()
            ])
            .current_dir(&node)
            .status()
            .unwrap()
            .success(),
        "Linking failed"
    );

    println!("cargo::rustc-link-search=native={}", lib.display());

    for dll in fs::read_dir(&bin).unwrap().filter_map(|d| {
        if let Some(d) = d.ok() {
            if d.file_name().to_string_lossy().ends_with(".dll") {
                return Some(d);
            }
        }
        return None;
    }) {
        let def = lib.join(dll.path().with_extension("def").file_name().unwrap());
        let imp = def.with_extension("lib");
        let libname = &*imp.file_stem().unwrap().to_string_lossy();

        println!("cargo::rustc-link-lib=dylib={libname}");

        if imp.exists() {
            continue;
        }

        fs::write(&def, dumpbin_exe(&dll.path())).unwrap();
        lib_exe(arch, &def, &imp);
    }

    #[cfg(windows)]
    os::windows::fs::symlink_dir(&bin, &out)
        .or_else(|err| {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(err)
            }
        })
        .unwrap();

    assert!(
        Command::new(RUNNER)
            .args([
                "--yes",
                PACK,
                "--preset",
                "win32",
                "--builtins",
                builtins.to_str().unwrap(),
                "--linked",
                "--out",
                bundle.to_str().unwrap(),
                entry.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success(),
        "Bundling failed"
    );

    out
}
