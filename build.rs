fn main() {
    // Find libweston via pkg-config
    let lib = pkg_config::Config::new()
        .probe("libweston-10")
        .expect("libweston-10 not found");

    // Tell Cargo to link it
    for path in lib.include_paths {
        println!("cargo:include={}", path.display());
    }
}

