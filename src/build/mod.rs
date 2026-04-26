#[cfg(target_os = "android")]
use std::fs;
use std::{env, path::PathBuf};

pub fn autolink() {
    let project = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project = project.parent().unwrap();
    let resource_dir = env::var("DEP_TAURI_PLUGIN_BARE_KIT_RESOURCE_DIR").unwrap();

    #[cfg(desktop)]
    env::set_var(
        "TAURI_CONFIG",
        format!("{{ \"bundle\": {{ \"resources\": {{ \"{resource_dir}\": \"\" }} }} }}"),
    );

    #[cfg(target_os = "android")]
    {
        println!("cargo::rustc-link-arg=-Wl,-rpath,$ORIGIN");

        for dir in fs::read_dir(resource_dir)
            .unwrap()
            .filter_map(|dir| dir.ok())
        {
            let file_type = dir.file_type().unwrap();

            if file_type.is_dir() {
                println!("cargo::rustc-link-search=native={}", dir.path().display());
            }
        }

        println!("cargo::rustc-link-lib=bare-kit");
    }

    #[cfg(target_vendor = "apple")]
    {
        println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/");
        println!("cargo::rustc-link-search=framework={resource_dir}");
        println!("cargo::rustc-link-lib=framework=BareKit");
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo::rustc-link-arg=-Wl,-rpath=$ORIGIN");
        println!("cargo::rustc-link-search=native={resource_dir}");
        println!("cargo::rustc-link-lib=bare-kit");
    }

    #[cfg(target_os = "windows")]
    {
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
