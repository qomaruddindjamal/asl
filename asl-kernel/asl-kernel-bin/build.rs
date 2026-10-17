fn main() {
    println!("cargo:rerun-if-env-changed=DEBUG");
    println!("cargo:rerun-if-env-changed=OPT_LEVEL");

    let is_debug = std::env::var("DEBUG").unwrap_or_default() == "true";
    let profile = if is_debug { "debug" } else { "release" };
    println!("cargo:rustc-env=ASL_BUILD_PROFILE={}", profile);
}


