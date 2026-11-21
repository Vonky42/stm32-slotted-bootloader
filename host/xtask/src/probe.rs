use crate::CHIP;
use crate::cargo::run;
use anyhow::{Context, Result};
use probe_rs::{Permissions, Session, SessionConfig, probe::list::Lister};
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

pub fn attach() -> anyhow::Result<Session> {
    Session::auto_attach(CHIP, SessionConfig::default()).with_context(|| {
        format!(
            "attaching to {} (make sure probe is connected and not in use)",
            CHIP
        )
    })
}

// UNTESTED
pub fn attach_under_reset() -> anyhow::Result<Session> {
    let probes = Lister::new().list_all();
    let probe = probes.first().context("no debug probe found")?.open()?;
    Ok(probe.attach(CHIP, Permissions::default())?)
}
