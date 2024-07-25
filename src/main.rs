use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Deserialize)]
struct Setting {
    hidden: Option<bool>,
    detach: Option<bool>,
    cmd: String,
}

fn main() {
    let exe_path = env::current_exe().unwrap();
    let exe_name = exe_path.file_stem().unwrap().to_owned().into_string().unwrap();
    let toml_path = exe_path.parent().unwrap().to_owned().join(exe_name + ".toml");

    let raw_toml = &fs::read_to_string(toml_path).unwrap();
    let setting: Setting = toml::from_str(raw_toml).unwrap();

    print!("{}", setting.cmd);
    print!("{}", setting.detach.unwrap_or(false));
    print!("{}", setting.hidden.unwrap_or(false));
}
