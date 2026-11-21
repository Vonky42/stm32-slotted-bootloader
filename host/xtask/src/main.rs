mod cargo;
mod probe;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

const CHIP: &str = "STM32F411RETx";
const BOOTLOADER: &str = "stm32-slotted-bootloader";

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build and flash onto the STM32
    Flash {
        #[command(subcommand)]
        target: FlashTarget,
    },
}

#[derive(Subcommand)]
enum FlashTarget {
    /// Builds and flashes the bootloader via probe-rs
    Bootloader {
        #[command(flatten)]
        build: BuildArgs,

        #[arg(long)]
        log: bool,
    },
    // App(FlashApp),
}

#[derive(Args, Debug)]
struct BuildArgs {
    /// Whether to compile with the '--release' flag
    #[arg(long, conflicts_with = "profile")]
    release: bool,

    /// Profile
    #[arg(long)]
    profile: Option<String>,

    /// Features
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
}

impl BuildArgs {
    /// Cargo's output directory for the compiled profile
    pub fn profile_dir(&self) -> &str {
        match (&self.profile, &self.release) {
            (Some(p), _) if p == "dev" => "debug",
            (Some(p), _) => p,
            (None, true) => "release",
            (None, false) => "debug",
        }
    }
}

// #[derive(Args)]
// struct FlashApp {
// nmae: String,
// slot: u8,
// via: Transport,
// build: BuildArgs,
// debug: DebugArgs,
// }

// #[derive(Clone, ValueEnum)]
// enum Transport {
//     Probe, // Probe-rs
//     Uart,  // UART
// }

fn main() -> Result<()> {
    let cli = Cli::parse().cmd;
    match cli {
        Cmd::Flash { ref target } => match target {
            FlashTarget::Bootloader { build, log } => flash_bootloader(&build, *log),
        },
    }
}

/// Flashses the bootloader
fn flash_bootloader(build_args: &BuildArgs, log: bool) -> Result<()> {
    let elf = cargo::build(BOOTLOADER, build_args)?;
    if log {
        probe::run_elf(&elf)
    } else {
        probe::download_elf(&elf)?;
        probe::reset()
    }
}
