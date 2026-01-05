const COMMANDS: &[&str] = &["ping", "get_file_descriptor", "get_file_name"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
