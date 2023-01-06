extern crate core;

use std::env;
use std::path;

use dotenvy::{dotenv, dotenv_iter};

fn main() {
    if let Ok(dotenv_path) = dotenv() {
        println!("cargo:rerun-if-changed={}", dotenv_path.display());

        for env_var in dotenv_iter().unwrap() {
            let (key, value) = env_var.unwrap();
            env::set_var(&key, &value);
            println!("cargo:rustc-env={}={}", &key, &value);
        }
    }

    if let Ok(version) = env::var("OTS_BUILD_VERSION") {
        println!("cargo:rustc-env=OTS_BUILD_VERSION={}", version);
    }

    if let Ok(profile) = env::var("ROCKET_PROFILE") {
        if profile != "debug" {
            return;
        }

        init_tls();
    }
}

fn init_tls() {
    let certs_path = path::Path::new("../server_ssl/default.crt");
    let key_path = path::Path::new("../server_ssl/default.key");

    if !certs_path.exists() || !key_path.exists() {
        println!("Private key and certificate were not found. These are required tls connection. See server-ssl/README.md");
        return;
    }

    println!(
        "cargo:rustc-env=ROCKET_TLS={{certs=\"{}\",key=\"{}\"}}",
        certs_path.to_str().unwrap(),
        &key_path.to_str().unwrap()
    );
}
