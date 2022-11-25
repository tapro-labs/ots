use dotenvy::{dotenv, dotenv_iter};

fn main() {
    if let Ok(dotenv_path) = dotenv() {
        println!("cargo:rerun-if-changed={}", dotenv_path.display());

        for env_var in dotenv_iter().unwrap() {
            let (key, value) = env_var.unwrap();
            println!("cargo:rustc-env={key}={value}");
        }
    }
}
