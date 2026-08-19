use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(unix)]
const NPX: &str = "npx";
#[cfg(windows)]
const NPX: &str = "npx.cmd";

pub fn autolink() {
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project = src.parent().unwrap();

    let entry = Some(project.join("bare/app.js"));
    let entry = if entry.as_ref().unwrap().exists() {
        entry
    } else {
        Some(project.join("bare/dist/app.js"))
    };
    let entry = if entry.as_ref().unwrap().exists() {
        entry
    } else {
        Some(project.join("bare/src/app.js"))
    };
    let entry = if entry.as_ref().unwrap().exists() {
        entry
    } else {
        Some(project.join("src-bare/app.js"))
    };
    let entry = if entry.as_ref().unwrap().exists() {
        entry
    } else {
        Some(project.join("src-bare/dist/app.js"))
    };
    let entry = if entry.as_ref().unwrap().exists() {
        entry
    } else {
        Some(project.join("src-bare/src/app.js"))
    };
    let entry = if entry.as_ref().unwrap().exists() {
        entry
    } else {
        None
    };

    let resource_dir = env::var("DEP_TAURI_PLUGIN_BARE_KIT_RESOURCE_DIR").unwrap();

    let platform = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let platform = match &*platform {
        "macos" => "darwin",
        "windows" => "win32",
        platform => platform,
    };
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = match &*arch {
        "arm" => "arm",
        "aarch64" => "arm64",
        "i686" => "ia32",
        "x86_64" => "x64",
        arch => panic!("Unsupported architecture: {arch}"),
    };

    if vec!["darwin", "linux", "win32"].contains(&platform) {
        env::set_var(
            "TAURI_CONFIG",
            format!(
                "{{ \"bundle\": {{ \"resources\": {{ \"{}\": \"\" }} }} }}",
                resource_dir.replace("\\", "\\\\"),
            ),
        );
    }

    if platform == "darwin" {
        let host = format!("{platform}-{arch}");

        assert!(Command::new(NPX)
            .current_dir(project)
            .args(vec![
                "--yes",
                "bare-link",
                "--host",
                &*host,
                "--out",
                &*resource_dir,
                project.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success());

        if let Some(entry) = &entry {
            assert!(Command::new(NPX)
                .current_dir(project)
                .args(vec![
                    "--yes",
                    "bare-pack",
                    "--host",
                    &*host,
                    "--linked",
                    "--out",
                    project.join("app.bundle.json").to_str().unwrap(),
                    entry.to_str().unwrap(),
                ])
                .status()
                .unwrap()
                .success());
        }

        println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/");
        println!("cargo::rustc-link-search=framework={resource_dir}");

        for framework in fs::read_dir(&resource_dir).unwrap().filter_map(|e| {
            if let Ok(e) = e {
                return if e.metadata().unwrap().is_dir() {
                    Some(e.file_name().to_string_lossy().to_string())
                } else {
                    None
                };
            }
            None
        }) {
            println!(
                "cargo::rustc-link-lib=framework={}",
                framework.strip_suffix(".framework").unwrap()
            );
        }
    }

    if platform == "ios" {
        let simulator = env::var("CARGO_CFG_TARGET_ENV").unwrap() == "sim";
        let host = format!(
            "{platform}-{arch}{}",
            if simulator { "-simulator" } else { "" }
        );
        let arch = match arch {
            "arm64" => "arm64",
            "x64" => "x86_64",
            arch => panic!("Unsupported architecture: {arch}"),
        };
        let profile = env::var("PROFILE").unwrap();
        let out = src.join(format!("gen/apple/Externals/{arch}/{profile}"));

        assert!(Command::new(NPX)
            .current_dir(project)
            .args(vec![
                "--yes",
                "bare-link",
                "--host",
                &*host,
                "--out",
                out.to_str().unwrap(),
                project.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success());

        if let Some(entry) = &entry {
            assert!(Command::new(NPX)
                .current_dir(project)
                .args(vec![
                    "--yes",
                    "bare-pack",
                    "--host",
                    &*host,
                    "--linked",
                    "--out",
                    project.join("app.bundle.json").to_str().unwrap(),
                    entry.to_str().unwrap(),
                ])
                .status()
                .unwrap()
                .success());
        }

        copy_dir_all(
            PathBuf::from(&*resource_dir).join("BareKit.framework"),
            out.join("BareKit.framework"),
        );

        println!("cargo::rustc-link-search=framework={}", out.display());
        println!("cargo::rustc-link-lib=framework=BareKit");
    }

    if platform == "android" {
        let host = format!("{platform}-{arch}");
        let arch = match &*arch {
            "arm" => "armeabi-v7a",
            "arm64" => "arm64-v8a",
            "ia32" => "x86",
            "x64" => "x86_64",
            arch => panic!("Unsupported architecture: {arch}"),
        };
        let out = src.join("gen/android/app/src/main/jniLibs");

        assert!(Command::new(NPX)
            .current_dir(project)
            .args(vec![
                "--yes",
                "bare-link",
                "--host",
                &*host,
                "--out",
                out.to_str().unwrap(),
                project.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success());

        if let Some(entry) = &entry {
            assert!(Command::new(NPX)
                .current_dir(project)
                .args(vec![
                    "--yes",
                    "bare-pack",
                    "--host",
                    &*host,
                    "--linked",
                    "--out",
                    project.join("app.bundle.json").to_str().unwrap(),
                    entry.to_str().unwrap(),
                ])
                .status()
                .unwrap()
                .success());
        }

        let out = out.join(arch);

        fs::copy(
            PathBuf::from(&*resource_dir).join("libbare-kit.so"),
            out.join("libbare-kit.so"),
        )
        .unwrap();
        fs::copy(
            PathBuf::from(&*resource_dir).join("libc++_shared.so"),
            out.join("libc++_shared.so"),
        )
        .unwrap();

        println!("cargo::rustc-link-search=native={}", out.display());
        println!("cargo::rustc-link-lib=bare-kit");
    }

    if platform == "linux" {
        let out = PathBuf::from(env::var("OUT_DIR").unwrap());
        let host = format!("{platform}-{arch}");

        assert!(Command::new(NPX)
            .current_dir(project)
            .args(vec![
                "--yes",
                "bare-link",
                "--host",
                &*host,
                "--out",
                out.to_str().unwrap(),
                project.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success());

        if let Some(entry) = &entry {
            assert!(Command::new(NPX)
                .current_dir(project)
                .args(vec![
                    "--yes",
                    "bare-pack",
                    "--host",
                    &*host,
                    "--linked",
                    "--out",
                    project.join("app.bundle.json").to_str().unwrap(),
                    entry.to_str().unwrap(),
                ])
                .status()
                .unwrap()
                .success());
        }

        copy_dir_all(out.join("lib"), PathBuf::from(&*resource_dir));

        println!("cargo::rustc-link-arg=-Wl,-rpath=$ORIGIN");
        println!("cargo::rustc-link-search=native={resource_dir}");

        for lib in fs::read_dir(&resource_dir).unwrap().filter_map(|e| {
            if let Ok(e) = e {
                let fname = e.file_name().to_string_lossy().to_string();

                return if fname.starts_with("lib") && fname.ends_with(".so") {
                    Some(fname)
                } else {
                    None
                };
            }
            None
        }) {
            let lname = lib
                .strip_prefix("lib")
                .unwrap()
                .strip_suffix(".so")
                .unwrap();

            println!("cargo::rustc-link-lib={lname}");
        }
    }

    if platform == "win32" {
        let host = format!("{platform}-{arch}");

        assert!(Command::new(NPX)
            .current_dir(project)
            .args(vec![
                "--yes",
                "bare-link",
                "--host",
                &*host,
                "--out",
                &*resource_dir,
                project.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success());

        if let Some(entry) = &entry {
            assert!(Command::new(NPX)
                .current_dir(project)
                .args(vec![
                    "--yes",
                    "bare-pack",
                    "--host",
                    &*host,
                    "--linked",
                    "--out",
                    project.join("app.bundle.json").to_str().unwrap(),
                    entry.to_str().unwrap(),
                ])
                .status()
                .unwrap()
                .success());
        }

        println!(
            "cargo:rustc-link-arg=/DEF:{}",
            PathBuf::from(&*resource_dir).join("bare-kit.def").display(),
        );
    }
}

fn copy_dir_all<P: AsRef<Path>>(src: P, dest: P) {
    fs::create_dir_all(&dest).unwrap();

    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ftype = entry.file_type().unwrap();
        if ftype.is_dir() {
            copy_dir_all(entry.path(), dest.as_ref().join(entry.file_name()));
        } else {
            fs::copy(entry.path(), dest.as_ref().join(entry.file_name())).unwrap();
        }
    }
}
