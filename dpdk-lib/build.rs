
use std::env;
use std::process;
use std::io::{Read, Write};
use std::path::{PathBuf};
use std::fs::{File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const LIB_DIR:&str = "src";
const DPDK_API:[&str; 3] = ["rte_build_config.h", "rte_ethdev.h", "rte_eal.h"];

/// bindgen 0.69.4 cannot translate functional macro expressions.
///
/// Substitutions:
/// RTE_BIT32(X) -> (1U<<X)
/// RTE_BIT64(X) -> (1UL<<X)
/// UINT64_C(X) ->  (XUL)

const C_MACRO_SUBSTITUTE_NUM:usize = 3;
const C_MACRO_SUBSTITUTE:[&str;C_MACRO_SUBSTITUTE_NUM] = [
    r#"s/RTE_BIT64\(([0-9]{1,})\)/\(1UL<<\1\)/"#,
    r#"s/RTE_BIT32\(([0-9]{1,})\)/\(1U<<\1\)/"#,
    r#"s/UINT64_C\(([0-9]{1,})\)/\(\1UL\)/"#,
];

fn patch_src_macro(filename:&str) {
    C_MACRO_SUBSTITUTE.iter().for_each(|pattern| {
        let _ = Command::new("sed")
            .arg("-i")
            .arg("-E")
            .arg(*pattern)
            .arg(filename)
            .output()
            .expect("sed execution failed");
    })
}

fn copy_file(dst:&str, src:&str) {
    let _ = Command::new("cp")
        .arg("-f")
        .arg(src)
        .arg(dst)
        .output()
        .expect("copy failed");
}

fn main() {
    let dpdk_home = validate_envir();
    let lib_dir = PathBuf::from(LIB_DIR);
    let mut lib = clean_all();

    let dpdk_include = dpdk_home.join("include");
    let clang_include_path = format!("-I{}", dpdk_include.to_str().unwrap());
    for include_file in DPDK_API {
        let input_file = dpdk_include.join(include_file);
        let input_file_patched = format!("{}_patched.h", include_file.replace(".h", ""));
        copy_file(&input_file_patched, input_file.to_str().unwrap());
        patch_src_macro(&input_file_patched);
        let output_file_path = lib_dir.join(include_file.replace(".h", ".rs"));
        let output_file_name = output_file_path.to_str().unwrap();
        bindgen::Builder::default()
            .clang_args(vec![&clang_include_path])
            .opaque_type("rte_l2tpv2_combined_msg_hdr|rte_gtp_psc_generic_hdr")
            .header(input_file_patched)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("bindings failed for rte_build_config.h")
            .write_to_file("external.rs")
            .expect("failed to write dpdk configuration");
        file_append(output_file_name, "external.rs");
        std::fs::set_permissions(output_file_name,
                                 Permissions::from_mode(0x124)).unwrap();
        std::fs::remove_file("external.rs").unwrap();
        let mod_entry = format!("pub mod {};\n", include_file.replace(".h", ""));
        let _ = lib.write(mod_entry.as_bytes());
    }
    println!("cargo:rerun-if-changed=build.rs");
}

fn validate_envir() -> PathBuf {
    let dpdk_home = match env::var("DPDK_HOME") {
        Ok(dir) => {
            println!("DPDK_HOME={dir}");
            PathBuf::from(dir)
        },
        Err(_) => {
            println!("cannot find DPDK_HOME environment");
            process::exit(255);
        }
    };
    match pkg_config::Config::new().probe("libdpdk") {
        Ok(_) => (),
        Err(err) => {
            println!("pkgconfig failed to locate \'libdpdk\' {:?}", err);
            process::exit(255);
        }
    }
    dpdk_home
}

fn clean_all() -> File {
    match std::fs::remove_dir_all(LIB_DIR) {
        Ok(_) => (),
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::NotFound => (),
                _ => panic!("failed to remote {}: {:?}", LIB_DIR, err)
            }
        }
    }
    match std::fs::create_dir(LIB_DIR) {
        Ok(_) => (),
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::AlreadyExists=> (),
                _ => panic!("failed to create \'{}\': {:?}", LIB_DIR, err)
            }
        }
    };
    File::create("src/lib.rs").unwrap()
}

fn file_append(dst:&str, src:&str) {
    let mut df = File::create(dst).unwrap();
    let mut sf = File::options()
        .read(true)
        .open(src).unwrap();

    let mut buffer:Vec<u8> = vec![];
    sf.read_to_end(&mut buffer).unwrap();
    df.write("#![allow(non_upper_case_globals)]\n".as_bytes()).unwrap();
    df.write("#![allow(non_camel_case_types)]\n".as_bytes()).unwrap();
    df.write("#![allow(non_snake_case)]\n\n".as_bytes()).unwrap();
    df.write("#![allow(warnings)] \n\n".as_bytes()).unwrap();
    df.write_all(buffer.as_slice()).unwrap();
}