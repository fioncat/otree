use std::env;
use std::error::Error;
use std::process::Command;

use simple_error::bail;
use vergen::{BuildBuilder, CargoBuilder, Emitter, RustcBuilder, SysinfoBuilder};

fn uncommitted_count() -> Result<usize, Box<dyn Error>> {
    let output = exec_git(&["status", "-s"])?;
    let lines = output.trim().split('\n');
    Ok(lines.filter(|line| !line.trim().is_empty()).count())
}

fn exec_git(args: &[&str]) -> Result<String, Box<dyn Error>> {
    let mut cmd = Command::new("git");
    let output = cmd.args(args).output()?;
    if !output.status.success() {
        let cmd = format!("git {}", args.join(" "));
        bail!("Execute git command {} failed", cmd);
    }
    let output = String::from_utf8(output.stdout)?;
    Ok(output.trim().to_string())
}

fn fetch_git_info() -> Result<(), Box<dyn Error>> {
    let describe = exec_git(&["describe", "--tags"]).unwrap_or_default();
    let sha = exec_git(&["rev-parse", "HEAD"])?;
    let short_sha = exec_git(&["rev-parse", "--short", "HEAD"])?;

    let cargo_version = env!("CARGO_PKG_VERSION");
    let stable_tag = format!("v{cargo_version}");
    let (mut version, mut build_type) = if stable_tag == describe {
        if cargo_version.ends_with("alpha") {
            (cargo_version.to_string(), "alpha")
        } else if cargo_version.ends_with("beta") {
            (cargo_version.to_string(), "beta")
        } else if cargo_version.ends_with("rc") {
            (cargo_version.to_string(), "pre-release")
        } else {
            (cargo_version.to_string(), "stable")
        }
    } else if describe.is_empty() {
        (cargo_version.to_string(), "stable")
    } else {
        (format!("{cargo_version}-dev_{short_sha}"), "dev")
    };

    let uncommitted_count = uncommitted_count()?;
    if uncommitted_count > 0 {
        version = format!("{version}-uncommitted");
        build_type = "dev-uncommitted";
    }

    println!("cargo:rustc-env=OTREE_VERSION={version}");
    println!("cargo:rustc-env=OTREE_BUILD_TYPE={build_type}");
    println!("cargo:rustc-env=OTREE_SHA={sha}");

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let rustc = RustcBuilder::all_rustc()?;
    let si = SysinfoBuilder::all_sysinfo()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;

    println!(
        "cargo:rustc-env=OTREE_TARGET={}",
        env::var("TARGET").unwrap()
    );

    // 始终尝试获取 git 信息，失败时使用默认值
    if let Err(_) = fetch_git_info() {
        let cargo_version = env!("CARGO_PKG_VERSION");
        println!("cargo:rustc-env=OTREE_VERSION={cargo_version}");
        println!("cargo:rustc-env=OTREE_BUILD_TYPE=stable");
        println!("cargo:rustc-env=OTREE_SHA=<unknown>");
    }

    Ok(())
}
