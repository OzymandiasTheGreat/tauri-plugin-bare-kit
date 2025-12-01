use std::{
    env, fs, os,
    path::{Path, PathBuf},
    process::Command,
};

const LINK: &str = "bare-link@2.1.10";
const PACK: &str = "bare-pack@1.5.1";
const PLUGIN: &str = "tauri_plugin_bare_kit";
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
        let relative = pathdiff::diff_paths(&out, &src).unwrap();
        let dest = match platform {
            "darwin" => "Frameworks",
            "linux" => "lib",
            _ => "",
        };

        env::set_var(
            "TAURI_CONFIG",
            format!(
                "{{ \"bundle\": {{ \"resources\": {{ \"{}\": \"{}\" }} }} }}",
                relative.display(),
                dest,
            ),
        );
    }
}

fn link_for_android<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    todo!("Link Android!");
}

fn link_for_darwin<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("Frameworks");
    let profile = env::var("PROFILE").unwrap();
    let temp = env::temp_dir().join(PLUGIN).join(profile);
    let dest = temp.join("Frameworks");
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
    todo!("Link iOS!");
}

fn link_for_linux<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    todo!("Link Linux!");
}

fn link_for_windows<P: AsRef<Path>>(src: &P) -> PathBuf {
    let src = src.as_ref();
    todo!("Link Windows!");
}
