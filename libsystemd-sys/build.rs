extern crate pkg_config;

fn main() {
    let _ = pkg_config::find_library("libsystemd");
}
