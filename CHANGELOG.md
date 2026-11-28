# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-27

### Added

#### Bootloader
- Bootloader for the STM32F411RE (Nucleo-F411RE) with four execute-in-place application slots (64 KB + 3 × 128 KB) behind a 32 KB bootloader region.
- Slot scan at reset; boots the first valid slot unless B1 is held.
- Boot menu: hold B1 at reset to enter. A short press cycles through valid slots (LD2 blinks the slot number), holding for 1 s arms the selected slot (LED solid), and releasing boots it. The LED flashes rapidly when no valid slot is found.
- Menu timeout: boots the selected slot after 30 s of inactivity.
- 20 ms button debouncing; timing based on the DWT cycle counter (no timers or interrupts).
- Clean handoff to the application: disables interrupts and SysTick, clears NVIC enable/pending bits, sets `VTOR` to the slot's vector table and re-enables interrupts before jumping.
- Optional `defmt` feature (default) for RTT logging and `panic-probe`; can be disabled to build without logging.

#### Slot images
- Slot header (`slot_header!()`) carrying the slot number, application name, version, image size and CRC-32s.
- Two-stage validation: header (magic, header CRC, slot number, size), then image (image CRC, initial stack pointer and reset vector). Rejections are reported as `SlotError` (`Empty`, `BadMagic`, `BadHeaderCrc`, `WrongSlot`, `BadSize`, `BadImageCrc`, `BadVectors`).
- Generated `memory.x` for the bootloader and for each application slot. No hand-written linker scripts.

#### Crates
- `boot-core`: shared by firmware and host tools. Contains the layout, header, CRC, validation,
  header patching, time types and `memory.x` generation.
- `boot-backend`: slot scanning, clock and `jump_to_slot` (Cortex-M specific).
- `boot-frontend`: hardware-independent menu state machine with the `Indicator` and `MenuInput` traits, plus the `LedIndicator` and `SingleButton` implementations.

#### Tooling
- `cargo xtask` commands: `flash bootloader [--log]`, `flash app <name> --slot <n>`,
  `erase --slot <n>` and `scan [--under-reset]`.
- `xtask` build options: `--release`, `--profile` and `--features`.
- `flash app` builds the application for its slot, converts the ELF to a bounds-checked binary, patches the header and flashes it via probe-rs.
- `scan` reads all slots from the host and runs the same validation as the bootloader. It shows each slot's name, version and size, or why the slot was rejected.

#### Examples
- `blinky`: toggles LD2 from a TIM2 interrupt.
- `pushy`: mirrors B1 onto LD2.

[Unreleased]: https://github.com/Vonky42/stm32-slotted-bootloader/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Vonky42/stm32-slotted-bootloader/releases/tag/v0.1.0