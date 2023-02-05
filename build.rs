use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

fn main() {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if os != "linux" {
        eprintln!("This crate is supported only for Linux targets.");
        std::process::exit(1);
    }

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let mut header_filename = std::path::PathBuf::from_str("data").unwrap();
    header_filename.push(&arch);
    header_filename.push("syscall.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rerun-if-changed={}",
        header_filename.to_string_lossy()
    );
    let raw_vars = match parse_syscall_header(&header_filename) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Error reading {:?}: {}.\n", &header_filename, err);
            eprintln!("This crate does not currently support {}.", &arch);
            std::process::exit(1);
        }
    };

    let mut out_file = std::path::PathBuf::from_str(&env::var("OUT_DIR").unwrap()).unwrap();
    out_file.push(format!("syscall_nrs_{}.rs", &arch));
    generate_constants_rs(out_file, &raw_vars).unwrap();
}

fn parse_syscall_header(filename: impl AsRef<Path>) -> std::io::Result<BTreeMap<String, u64>> {
    use std::io::BufRead;

    let filename = filename.as_ref();
    let f = File::open(filename)?;
    let mut vars: BTreeMap<String, u64> = BTreeMap::new();
    for line in BufReader::new(f).lines() {
        let line = line?;
        if !line.starts_with("#define ") {
            continue;
        }
        let (_, line) = line.split_at(8);
        let (var_name, expr) = line.split_once(' ').unwrap();
        let v = eval_expr(expr, &vars);
        vars.insert(var_name.to_string(), v);
    }
    Ok(vars)
}

fn generate_constants_rs(
    filename: impl AsRef<Path>,
    raw_vars: &BTreeMap<String, u64>,
) -> std::io::Result<()> {
    use std::io::Write;

    let mut f = File::create(filename)?;
    for (k, v) in raw_vars.iter() {
        if k.starts_with("__NR_") {
            let (_, name) = k.split_at(5);
            writeln!(
                f,
                "/// The system call number for `{}` on this platform.",
                name
            )?;
            // We also define an extra configuration pair for each system
            // call name, so that we can adapt the calls to wrappers to
            // only include those that make sense for the current platform.
            println!("cargo:rustc-cfg=have_syscall={:?}", name);
            let name = name.to_uppercase();
            writeln!(f, "pub const {}: V = {};", name, v)?;
        }
    }
    Ok(())
}

fn eval_expr(expr: &str, _vars: &BTreeMap<String, u64>) -> u64 {
    // TODO: Implement the bare minimum possible parser for the expressions
    // in all of musl's syscall.h files.
    expr.parse().unwrap()
}
