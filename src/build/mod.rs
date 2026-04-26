use std::{env, path::PathBuf};

pub fn autolink() {
    let project = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project = project.parent().unwrap();
    let resource_dir = env::var("DEP_TAURI_PLUGIN_BARE_KIT_RESOURCE_DIR").unwrap();
    let platform = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if vec!["macos", "linux", "windows"].contains(&&*platform) {
        env::set_var(
            "TAURI_CONFIG",
            format!("{{ \"bundle\": {{ \"resources\": {{ \"{resource_dir}\": \"\" }} }} }}"),
        );
    }

    if platform == "android" {
        println!("cargo::rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo::rustc-link-search=native={resource_dir}");
        println!("cargo::rustc-link-lib=bare-kit");
    }

    if vec!["ios", "macos"].contains(&&*platform) {
        println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/");
        println!("cargo::rustc-link-search=framework={resource_dir}");
        println!("cargo::rustc-link-lib=framework=BareKit");
    }

    if platform == "linux" {
        println!("cargo::rustc-link-arg=-Wl,-rpath=$ORIGIN");
        println!("cargo::rustc-link-search=native={resource_dir}");
        println!("cargo::rustc-link-lib=bare-kit");
    }

    if platform == "windows" {
        println!("cargo::rustc-link-search=native={resource_dir}\\lib");
        println!("cargo::rustc-link-lib=dylib=bare-kit");
    }

    println!("cargo::rerun-if-changed={}", project.join("bare").display());
    println!(
        "cargo::rerun-if-changed={}",
        project.join("src-bare").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project.join("common").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project.join("src-common").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project.join("app.bundle.json").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project.join("app.bundle.json.d").display()
    );
}
