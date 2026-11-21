use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::BuildArgs;

const TARGET: &str = "thumbv7em-none-eabihf";

/// Returnt the firmware directory (this is hardcoded)
pub fn firmware_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../firmware")
}

/// Build `package` in the firmware workspace and return the path to its ELF
pub fn build(package: &str, args: &BuildArgs) -> Result<PathBuf> {
    let dir = firmware_dir();
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&dir);
    cmd.args(["build", "-p", package]);

    if args.release {
        cmd.arg("--release");
    }

    if let Some(profile) = &args.profile {
        cmd.args(["--profile", &profile]);
    }

    if !args.features.is_empty() {
        cmd.args(["--features", &args.features.join(",")]);
    }

    run(&mut cmd)?;

    Ok(dir
        .join("target")
        .join(TARGET)
        .join(args.profile_dir())
        .join(package))
}

/// Run the command
pub fn run(cmd: &mut Command) -> Result<()> {
    let status = cmd
        .status()
        .with_context(|| format!("faile to run {:?} (is it installed?)", cmd.get_program()))?;
    if !status.success() {
        bail!("{:?} failed with {status}", cmd.get_program());
    }

    Ok(())
}
