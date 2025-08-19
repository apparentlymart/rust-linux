use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if os != "linux" {
        eprintln!("This crate is supported only for Linux targets.");
        std::process::exit(1);
    }

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    {
        let mut header_filename = std::path::PathBuf::from_str("data").unwrap();
        header_filename.push(&arch);
        header_filename.push("syscall.h");
        println!(
            "cargo:rerun-if-changed={}",
            header_filename.to_string_lossy()
        );
        let raw_vars = match parse_header(&header_filename) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Error reading {:?}: {}.\n", &header_filename, err);
                eprintln!("This crate does not currently support {}.", &arch);
                std::process::exit(1);
            }
        };

        let mut out_file = std::path::PathBuf::from_str(&env::var("OUT_DIR").unwrap()).unwrap();
        out_file.push(format!("syscall_nrs_{}.rs", &arch));
        generate_syscall_constants_rs(out_file, &raw_vars).unwrap();
    }
    {
        let mut header_filename = std::path::PathBuf::from_str("data").unwrap();
        header_filename.push(&arch);
        header_filename.push("errno.h");
        println!(
            "cargo:rerun-if-changed={}",
            header_filename.to_string_lossy()
        );
        let raw_vars = match parse_header(&header_filename) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Error reading {:?}: {}.\n", &header_filename, err);
                eprintln!("This crate does not currently support {}.", &arch);
                std::process::exit(1);
            }
        };

        let mut out_file = std::path::PathBuf::from_str(&env::var("OUT_DIR").unwrap()).unwrap();
        out_file.push(format!("errnos_{}.rs", &arch));
        generate_errno_constants_rs(out_file, &raw_vars).unwrap();
    }
}

fn parse_header(filename: impl AsRef<Path>) -> std::io::Result<BTreeMap<String, u64>> {
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
        let v = eval_expr(expr);
        let v = match v {
            NumOrAlias::Num(v) => v,
            NumOrAlias::Alias(name) => *(vars.get(&name).unwrap()),
        };
        vars.insert(var_name.to_string(), v);
    }
    Ok(vars)
}

fn generate_syscall_constants_rs(
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
    println!("cargo::rustc-check-cfg=cfg(have_syscall, values(any()))");
    Ok(())
}

fn generate_errno_constants_rs(
    filename: impl AsRef<Path>,
    raw_vars: &BTreeMap<String, u64>,
) -> std::io::Result<()> {
    use std::io::Write;

    let mut f = File::create(filename)?;
    for (k, v) in raw_vars.iter() {
        if k.starts_with("E") {
            writeln!(f, "/// The error number for `{}` on this platform.", k)?;
            let name = k.to_uppercase();
            writeln!(f, "pub const {}: i32 = {};", name, v)?;
        }
    }

    writeln!(
        f,
        "/// Macro to help higher-level crates generate derived errno constants.",
    )?;
    writeln!(f, "#[doc(hidden)]")?;
    writeln!(f, "#[macro_export]")?;
    writeln!(f, "macro_rules! errno_derived_consts {{")?;
    writeln!(f, "    ($typ:ty, $transformfn:ident) => {{")?;
    for (k, _) in raw_vars.iter() {
        if k.starts_with("E") {
            let name = k.to_uppercase();
            writeln!(
                f,
                "        pub const {name}: $typ = $transformfn(::linux_unsafe::result::{name});"
            )?;
        }
    }
    writeln!(f, "    }};")?;
    writeln!(f, "}}")?;
    writeln!(f, "#[doc(hidden)]")?;
    writeln!(f, "pub use errno_derived_consts;")?;

    Ok(())
}

enum NumOrAlias {
    Num(u64),
    Alias(String),
}

fn eval_expr(expr: &str) -> NumOrAlias {
    if let Ok(v) = expr.parse::<u64>() {
        NumOrAlias::Num(v)
    } else {
        NumOrAlias::Alias(expr.to_string())
    }
}
