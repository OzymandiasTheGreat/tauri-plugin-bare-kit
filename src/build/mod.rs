use std::env;

pub fn autolink() {
    let resource_dir = env::var("DEP_TAURI_PLUGIN_BARE_KIT_RESOURCE_DIR").unwrap();

    env::set_var(
        "TAURI_CONFIG",
        format!("{{ \"bundle\": {{ \"resources\": {{ \"{resource_dir}\": \"\" }} }} }}"),
    );

    println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/");
    println!("cargo::rustc-link-search=framework={resource_dir}");
    println!("cargo::rustc-link-lib=framework=BareKit");
}
