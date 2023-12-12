#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::{
    ptr::write_bytes,
    sync::atomic::{AtomicBool, Ordering},
};

use arduino_hal::{Delay, Peripherals};
use i2c_slave::*;
use panic_halt as _;
use ufmt::{uwrite, uwriteln};

mod i2c_slave;

static TWI_INT_FLAG: AtomicBool = AtomicBool::new(false);

// I2C interrupt handler
#[avr_device::interrupt(atmega328p)]
fn TWI() {
    avr_device::interrupt::free(|_| {
        TWI_INT_FLAG.store(true, Ordering::SeqCst);
    });
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);

    let mut led = pins.d13.into_output();

    // Using external pullup resistors, so pins configured as floating inputs
    let sda = pins.a4.into_floating_input();
    let scl = pins.a5.into_floating_input();

    let slave_address: u8 = 0x26;

    let mut i2c_slave: I2cSlave = I2cSlave::new(dp.TWI, slave_address, sda, scl, &TWI_INT_FLAG);

    // Enable global interrupt
    unsafe { avr_device::interrupt::enable() };

    // Disabling power reduction for TWI
    dp.CPU.prr.write(|w| w.prtwi().clear_bit());

    // Value recieved from I2C Master
    let mut buf: [u8; 4];

    ufmt::uwriteln!(&mut serial, "Initialized with addr: 0x{:X}", slave_address).unwrap();

    led.set_low();

    loop {
        buf = [0; 4];
        i2c_slave.init(false);

        // RECEIVE
        match i2c_slave.receive(&mut buf) {
            Ok(_) => {
                uwrite!(&mut serial, "Received: ").unwrap();

                buf.iter().for_each(|b| {
                    uwrite!(&mut serial, "{} ", *b).unwrap();
                });
                uwrite!(&mut serial, "\n").unwrap();
            }
            Err(err) => {
                uwriteln!(&mut serial, "Error: {:?}", err).unwrap();
            }
        };

        // RESPOND

        // Multiply each u8 by 10 just to see difference in master's output clearly
        for b in buf.iter_mut() {
            *b *= 10;
        }

        match i2c_slave.respond(&mut buf) {
            Ok(count) => uwriteln!(&mut serial, "{} bytes has been sent back", count).unwrap(),
            Err(err) => uwriteln!(&mut serial, "{:?}", err).unwrap(),
        };

        uwrite!(&mut serial, "\n").unwrap();
    }
}
