use crate::CHIP;
use crate::cargo::run;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn download_elf(elf: &Path) -> Result<()> {
    run(Command::new("probe-rs")
        .args(["download", "--chip", CHIP])
        .arg(elf))
}

pub fn reset() -> Result<()> {
    run(Command::new("probe-rs").args(["reset", "--chip", CHIP]))
}

pub fn run_elf(elf: &Path) -> Result<()> {
    run(Command::new("probe-rs")
        .args(["run", "--chip", CHIP])
        .arg(elf))
}

pub fn download_bin(bin_path: &Path, base: u32) -> Result<()> {
    run(Command::new("probe-rs")
        .args([
            "download",
            "--chip",
            CHIP,
            "--binary-format",
            "bin",
            "--base-address",
            &format!("{base:#x}"),
        ])
        .arg(bin_path))
}
