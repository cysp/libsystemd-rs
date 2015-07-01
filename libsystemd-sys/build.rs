extern crate gcc;
extern crate pkg_config;

fn main() {
    if let Ok(libsystemd_pc) = pkg_config::find_library("libsystemd") {
        let mut c = gcc::Config::new();
        for include_path in libsystemd_pc.include_paths {
            c.include(include_path);
        }
        c.file("src/libsystemd_sys_helpers.c");
        c.compile("libsystemd_sys_helpers.a");
    }
}
