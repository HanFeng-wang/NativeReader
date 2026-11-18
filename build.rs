use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/native-reader.rc");
    println!("cargo:rerun-if-changed=assets/native-reader.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "x64",
        Ok("x86") => "x86",
        Ok("aarch64") => "arm64",
        _ => return,
    };
    let Some(rc) = find_resource_compiler(arch) else {
        println!("cargo:warning=rc.exe not found; the executable icon will not be embedded");
        return;
    };
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let resource = out_dir.join("native-reader.res");
    let status = Command::new(rc)
        .current_dir("assets")
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg("native-reader.rc")
        .status()
        .expect("failed to start rc.exe");
    if !status.success() {
        panic!("rc.exe failed with status {status}");
    }
    println!("cargo:rustc-link-arg-bins={}", resource.display());
}

fn find_resource_compiler(arch: &str) -> Option<PathBuf> {
    for variable in ["WindowsSdkDir", "ProgramFiles(x86)", "ProgramFiles"] {
        let Some(root) = env::var_os(variable) else {
            continue;
        };
        let root = PathBuf::from(root);
        let base = if variable == "WindowsSdkDir" {
            root.join("bin")
        } else {
            root.join("Windows Kits").join("10").join("bin")
        };
        let Ok(entries) = std::fs::read_dir(&base) else {
            continue;
        };
        let mut versions: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();
        versions.sort_by(|left, right| right.cmp(left));
        for version in versions {
            let candidate = version.join(arch).join("rc.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join("rc.exe"))
        .find(|candidate| candidate.is_file())
}
