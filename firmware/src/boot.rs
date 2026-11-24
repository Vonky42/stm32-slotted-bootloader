use boot_layout::Image;
use cortex_m::asm::{self, bootload};

/// Performs a bootload jump to the provided `slot`
pub fn jump_to_slot(image: Image) -> ! {
    match image {
        Image::Bootloader(_) => {
            panic!("cannot jump to the bootloader")
        }
        Image::Application(slot) => {
            unsafe {
                // Disable interrupts and clear pending interrupts
                debug!("Bootloader: disabling interrupts");
                cortex_m::interrupt::disable();
                let mut cp = cortex_m::Peripherals::steal();
                cp.SYST.disable_counter();
                cp.SYST.disable_interrupt();
                cp.SCB.icsr.write(1 << 25);

                debug!("Bootloader: clearing NVIC registers");
                for icer in cp.NVIC.icer.iter() {
                    icer.write(0xFFFF_FFFF);
                }
                for icpr in cp.NVIC.icpr.iter() {
                    icpr.write(0xFFFF_FFFF);
                }

                // Update Vector Table Offset Register (VTOR), re-enable interrupts, and perform jump
                debug!("Bootloader: Updating VTOR, enabling interrupts, and performing boot jump");
                let application = slot.application_region();
                cp.SCB.vtor.write(application.base);
                asm::dsb();
                asm::isb();
                cortex_m::interrupt::enable();
                bootload(application.base as *const u32);
            }
        }
    }
}
