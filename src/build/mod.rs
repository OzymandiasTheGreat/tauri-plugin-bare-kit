use std::{env, path::PathBuf};

pub fn autolink() {
    let project = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let resource_dir = env::var("DEP_TAURI_PLUGIN_BARE_KIT_RESOURCE_DIR").unwrap();

    env::set_var(
        "TAURI_CONFIG",
        format!("{{ \"bundle\": {{ \"resources\": {{ \"{resource_dir}\": \"\" }} }} }}"),
    );

    println!(
        "cargo::rerun-if-changed={}",
        project.parent().unwrap().join("bare").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project.parent().unwrap().join("src-bare").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project.parent().unwrap().join("common").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project.parent().unwrap().join("src-common").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project.parent().unwrap().join("app.bundle.json").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        project
            .parent()
            .unwrap()
            .join("app.bundle.json.d")
            .display()
    );
    println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/");
    println!("cargo::rustc-link-search=framework={resource_dir}");
    println!("cargo::rustc-link-lib=framework=BareKit");
}
