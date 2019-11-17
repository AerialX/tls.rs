use std::{env, fs::{self, File}};
use std::path::PathBuf;
use handlebars::Handlebars;
use serde::Serialize;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    println!("cargo:rustc-link-search={}", out.display());

    // TODO: also consider OS
    let asm_name = match &target {
        target if target.starts_with("thumbv") => "asm.s",
        target => panic!("unsupported target {}", target),
    };

    #[derive(Serialize)]
    struct TlsData {
        tb_pow2: usize,
    }

    let tls_data = TlsData {
        tb_pow2: 5,
    };

    let mut reg = Handlebars::new();
    reg.set_strict_mode(true);
    for f in &[asm_name, "tb.rs", "tb.ld"] {
        let src = PathBuf::from(format!("src/{}", f));
        let dest = out.join(format!("tls.{}", src.extension().unwrap().to_string_lossy()));
        println!("cargo:rerun-if-changed={}", src.display());

        let mut src = File::open(src).unwrap();
        let dest = File::create(out.join(dest)).unwrap();
        reg.render_template_source_to_write(&mut src, &tls_data, dest).unwrap();
    }
}
