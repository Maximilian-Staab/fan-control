#![no_std]
#![no_main]

#[macro_use]
mod macros;

#[cfg(not(feature = "serial"))]
use panic_halt as _;

mod commons;
use crate::commons::Controller;

#[cfg(target_vendor = "tiny85")]
mod attiny;
#[cfg(target_vendor = "tiny85")]
use crate::attiny::*;

#[cfg(target_vendor = "mega2560")]
mod mega2560;
#[cfg(target_vendor = "mega2560")]
use crate::mega2560::*;

#[cfg(feature = "serial")]
#[cfg(not(doc))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // code from the avr-hal examples
    // disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // get the peripherals so we can access serial and the LED.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // Print out panic location
    ufmt::uwriteln!(&mut serial, "Firmware panic!\r").void_unwrap();
    if let Some(loc) = info.location() {
        ufmt::uwriteln!(
            &mut serial,
            "  At {}:{}:{}\r",
            loc.file(),
            loc.line(),
            loc.column(),
        )
        .void_unwrap();
    }

    // Blink LED rapidly
    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(100);
    }
}

#[avr_device::entry]
fn main() -> ! {
    let mut controller = Board::new(25000);
    loop {
        arduino_hal::delay_ms(100);
        controller.update();
    }
}
