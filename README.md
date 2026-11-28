<div align="center">

# stm32-slotted-bootloader

**A bare-metal, `no_std` Rust bootloader for the STM32F411RE that hosts up to four independent firmware images and lets you pick which one to boot.**

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?logo=rust)
![Target](https://img.shields.io/badge/target-thumbv7em--none--eabihf-blue)
![Board](https://img.shields.io/badge/board-Nucleo--F411RE-03234B?logo=stmicroelectronics)
![Status](https://img.shields.io/badge/status-work_in_progress-yellow)

</div>

---

## Overview

The bootloader sits in the first 32KB of flash and manages four applications slots that are executed **in-place**. Each application is ordinary `cortex-m-rt` firmware, linked for its slot and tagged with a small header. On reset the bootloader validates every slot and boots the first valid one. Holding the user button while resetting enters the boot menu. Here you can select which slot to boot using the user button to cycle through the valid slots.

- **4 slots**, each with its own header (name, version, size, CRC-32)
- **Two-stage validation**: header CRC, slot number, and size first then image CRC and vector-table
- **LED + button menu**: debounced input, short press = next slot, long press = boot
- **`cargo xtask`**: build, flash, and scan applications from the host
- **Extensible and configurable**: plug in your own indicators and inputs through the `Indicator` and `MenuInput` traits, or change the slot layout ([details](#extending-the-bootloader)) 
- **Minimal changes to your application**: two lines turn a normal `cortex-m-rt` application into a slot image ([details](#writing-an-application))

## Getting started

**Requirements:** Rust (stable) with the `thumbv7em-none-eabihf` target, [`probe-rs`](https://probe.rs), a Nucleo-F411RE.

```sh
rustup target add thumbv7em-none-eabihf
cargo install probe-rs-tools
```

**Flash the bootloader and an application:**

```sh
cargo xtask flash bootloader            # add --log to stay attached for defmt output
cargo xtask flash app blinky --slot 0
cargo xtask flash app pushy  --slot 1
```

**Check what is in flash:**

```sh
cargo xtask scan
# slot 0: valid 'blinky' v0.1.0 (… B of 65536 B (… %))
# slot 1: valid 'pushy' v0.1.0 (…)
# slot 2: Empty
# slot 3: Empty
```

**Other commands:** `cargo xtask erase --slot <n>`, `scan --under-reset`, `--release` / `--profile` / `--features` on any build.

> **Note:** A plain `probe-rs download` of an app ELF will **not** boot: its header is unpatched, so the bootloader rejects it. Always flash apps through `xtask`.

## Using the boot menu

On reset, the bootloader checks all four slots and boots the first valid one. To choose a different slot, hold the user button (B1) while pressing reset. This opens the boot menu, where the green LED (LD2) blinks *n* times to show that slot *n* is selected.

| Input                    | Effect                                             |
|--------------------------|----------------------------------------------------|
| Short press              | Select the next valid slot                         |
| Hold for 1 s, release    | Boot the selected slot (LED stays on while held)   |
| No input for 30 s        | Boot the selected slot                             |

If no valid slot is detected, the LED flashes rapidly and the bootloader stays in the menu.

## Writing an application

1. Add a 'build.rs' that calls `boot_core::emit_application_memory_x_from_env()`.
2. Add `boot_core::slot_header!();` once in `main.rs`.
3. Flash it with `cargo xtask flash app <name> --slot <n> [OPTIONS]`

`xtask` builds the application for the given slot, converts the ELF to a binary, writes the size and CRCs into the header, and flashes it at the slot's base address.

**That's all an app needs.** It is still a normal `cortex-m-rt` binary that can use any HAL, the bare PAC, interrupts or RTIC. There is no bootloader-specific startup code and no runtime dependency on the bootloader: `boot-core` only adds a constant header and the generated `memory.x`, and the bootloader sets up `VTOR` and the stack pointer before jumping. An existing project needs two lines added to run from a slot.

## How it works

### Flash layout

| Sectors | Address       | Size        | Use                                  |
|:-------:|---------------|-------------|--------------------------------------|
| 0–1     | `0x0800_0000` | 32 KB       | Bootloader                           |
| 2–3     | `0x0800_8000` | 2 × 16 KB   | *Reserved: WIP*                      |
| 4       | `0x0801_0000` | 64 KB       | Slot 0                               |
| 5       | `0x0802_0000` | 128 KB      | Slot 1                               |
| 6       | `0x0804_0000` | 128 KB      | Slot 2                               |
| 7       | `0x0806_0000` | 128 KB      | Slot 3                               |

> ⚠️ **MCU-specific:** This layout is specific to the **STM32F411RE** (512 KB flash, 128 KB RAM). The slot sizes follow its uneven sector map (4 × 16 KB, then 64 KB, then 3 × 128 KB). See [Extending the bootloader](#extending-the-bootloader) to change it.

Each slot's application starts with a `0x200`-byte header, followed by the vector table. Both the header and the linker scripts (`memory.x`) are generated from `boot-core`. Keep in mind that your application will not compile if it does not fit in the slot.

### Repository layout
```
.
├── firmware/                   # thumbv7em-none-eabihf workspace
│   ├── bootloader/             # the bootloader binary
│   ├── crates/
│   │   ├── boot-core/          # layout, SlotHeader, CRC, patching, validation, time (no_std)
│   │   ├── boot-backend/       # slot scan, clock, jump_to_slot (Cortex-M specific)
│   │   └── boot-frontend/      # menu state machine, debouncer, LED indicator (HAL-agnostic)
│   └── applications/
│       ├── blinky/             # toggles LD2 from a TIM2 interrupt
│       └── pushy/              # mirrors the B1 button onto LD2
└── host/                       # host workspace (std)
    └── xtask/                  # build / patch / flash / erase / scan
```

## Extending the bootloader

The bootloader keeps the menu logic separate from the hardware, so most changes touch a single file or add a new type.

**Custom indicators.** The menu doesn't know what the LED is. It drives anything that implements `boot_frontend::Indicator`:

```rust
pub trait Indicator {
    fn show_selection(&mut self, slot: u8, info: &Result<SlotHeader, SlotError>, now: Instant);
    fn show_boot_pending(&mut self, now: Instant);
    fn show_error(&mut self, now: Instant);
    fn tick(&mut self, now: Instant);
}
```

`LedIndicator` is the built-in implementation. A UART log or a small display would be other implementations. `show_selection` also receives the slot's header, so a richer output can show the app name and version.

**Custom inputs.** Input works the same way. Any `boot_frontend::MenuInput` turns raw input into `Next`, `Arm` and `Boot` commands:

```rust
pub trait MenuInput {
    fn poll(&mut self, now: Instant) -> Option<MenuCommand>;
}
```

`SingleButton` (debounced, short and long press) is the built-in implementation. Two buttons, a rotary encoder or UART commands would just be other `MenuInput` types.

**Slot layout.** `firmware/crates/boot-core/src/layout.rs` is the reference point for the bootloader region, RAM, header composition, and the application slots. To use different slot sizes or another STM32, edit this file. For a different chip, also update the clock and pin setup in `bootloader/src/board.rs`.

## Status

The bootloader boots validated slots and has a working LED/button menu.

**Planned**
- UART text output mirroring the boot menu
- UART flashing / updating through a host `flasher` tool
- Persistent boot config (sectors 2 and 3), swap-style
- Let applications reboot into bootloader (`boot-core` API)
- Power-fail-safe in-field firmware UART updates

---

<sub>A learning project for bare-metal Rust on STM32, built on <code>stm32f4xx-hal</code> 0.23, <code>cortex-m-rt</code>, and <code>probe-rs</code>.</sub>