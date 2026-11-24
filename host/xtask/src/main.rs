mod cargo;
mod probe;
mod scan;

use anyhow::{Context, Result, anyhow, bail};
use boot_core::{HEADER_SIZE, SLOT_COUNT, Slot, patch};
use clap::{Args, Parser, Subcommand, ValueEnum};
use object::{
    Endianness, elf,
    read::elf::{ElfFile32, ProgramHeader},
};
use std::path::Path;

use crate::scan::scan_slots;

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
    /// Erase an application slot
    Erase {
        #[arg(long, value_parser = parse_slot)]
        slot: Slot,
    },
    /// Scan the application slots
    Scan {
        #[arg(long)]
        under_reset: bool,
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
    App(FlashApp),
}

#[derive(Args, Debug, Clone)]
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

#[derive(Args)]
struct FlashApp {
    name: String,

    #[arg(long, value_parser = parse_slot)]
    slot: Slot,

    #[arg(long, value_enum, default_value_t = Transport::Probe)]
    via: Transport,

    #[command(flatten)]
    build: BuildArgs,
}

fn parse_slot(s: &str) -> Result<Slot, String> {
    let index: usize = s.parse().map_err(|_| format!("`{s}` is not a number"))?;
    Slot::from_index(index).ok_or_else(|| format!("no slot {index} (0..={})", SLOT_COUNT - 1))
}

#[derive(Clone, ValueEnum)]
enum Transport {
    Probe, // Probe-rs
    Uart,  // UART
}

fn main() -> Result<()> {
    let cli = Cli::parse().cmd;
    match cli {
        Cmd::Flash { ref target } => match target {
            FlashTarget::Bootloader { build, log } => flash_bootloader(&build, *log),
            FlashTarget::App(application) => flash_app(application),
        },
        Cmd::Erase { slot } => erase_slot(&slot),
        Cmd::Scan { under_reset } => scan_slots(under_reset),
    }
}

/// Flashses the bootloader
fn flash_bootloader(build_args: &BuildArgs, log: bool) -> Result<()> {
    let elf = cargo::build(BOOTLOADER, build_args, &[])?;
    if log {
        probe::run_elf(&elf)
    } else {
        probe::download_elf(&elf)?;
        probe::reset()
    }
}

/// Flashes an applcation
fn flash_app(application: &FlashApp) -> Result<()> {
    let elf = cargo::build(
        &application.name,
        &application.build,
        &[("BOOT_SLOT", application.slot.index.to_string())],
    )?;
    let bin_path = elf.with_extension(format!("slot{}.bin", application.slot.index.to_string()));

    let mut bin = elf_to_bin(&elf, application.slot)?;

    let (header, image) = bin
        .split_first_chunk_mut::<{ HEADER_SIZE as usize }>()
        .context("binary is shorter than the slot header")?;
    patch(header, image);

    std::fs::write(&bin_path, &bin)?;
    println!(
        "{}: built for slot {}, {} B -> {:#010x}",
        application.name,
        application.slot.index,
        bin.len(),
        application.slot.region.base
    );

    match application.via {
        Transport::Probe => {
            probe::download_bin(&bin_path, application.slot.region.base)?;
            probe::reset()
        }
        Transport::Uart => {
            bail!("UART flashing has not been implemented yet! Please use Probe-rs using ST-LINK")
        }
    }
}

fn elf_to_bin(elf_path: &Path, slot: Slot) -> Result<Vec<u8>> {
    let data =
        std::fs::read(elf_path).with_context(|| format!("reading {}", elf_path.display()))?;
    let data = &*data;

    let elf = ElfFile32::<Endianness>::parse(data)?;
    let e = elf.endian();
    let mut bin = Vec::new();

    for program_header in elf.elf_program_headers() {
        if program_header.p_type(e) != elf::PT_LOAD || program_header.p_filesz(e) == 0 {
            continue;
        }

        let addr = program_header.p_paddr(e);
        let bytes = program_header
            .data(e, data)
            .map_err(|_| anyhow!("bad segment at {addr:#x}"))?;
        let end = addr + bytes.len() as u32;
        if addr < slot.region.base || end > slot.region.end() {
            bail!(
                "segment {:#010x}..{:#010x} is outside the slot boundaries {:#010x}..{:#010x}",
                addr,
                end,
                slot.region.base,
                slot.region.end(),
            );
        }

        let offset = (addr - slot.region.base) as usize;
        // Patch gaps with 0xFF (ERASED bytes)
        if bin.len() < offset + bytes.len() {
            bin.resize(offset + bytes.len(), 0xFF);
        }
        bin[offset..offset + bytes.len()].copy_from_slice(bytes);
    }

    if bin.len() < HEADER_SIZE as usize + 8 {
        bail!("no image found after the header. Make sure slot_header!() is used in the app!");
    }

    Ok(bin)
}

/// Erases `slot` by flashing a single ERASED (0xFF) byte to it. Each flash is preceded by a complete sector erasion, thus this erases the whole sector / empties the slot
fn erase_slot(slot: &Slot) -> Result<()> {
    let bin_path = cargo::firmware_dir()
        .join("target")
        .join("thumbv7em-none-eabihf")
        .join(format!("erase-slot{}.bin", slot.index));

    // STM32 erases the whole sector before flashing. A single ERASED byte (0xFF) write will thus cause hardware erase and cost only 1 byte of programming
    std::fs::write(&bin_path, vec![0xFF])?;

    println!(
        "erasing slot {} ({:#010x}, {} KB)",
        slot.index,
        slot.region.base,
        slot.region.length / 1024
    );

    probe::download_bin(&bin_path, slot.region.base)?;
    probe::reset()
}
