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

static TWI_INT_FLAG: AtomicBool = AtomicBool::new(false);

mod i2c_slave;

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

    let sda = pins.a4.into_floating_input();
    let scl = pins.a5.into_floating_input();

    let addr: u8 = 0x26;

    let mut i2c_slave: I2cSlave = I2cSlave::new(dp.TWI, addr, sda, scl, &TWI_INT_FLAG);

    // Enable global interrupt
    unsafe { avr_device::interrupt::enable() };

    // Disabling power reduction for TWI
    dp.CPU.prr.write(|w| w.prtwi().clear_bit());

    // Value recieved from I2C Master
    let mut buf: [u8; 4];

    ufmt::uwriteln!(&mut serial, "Initialized with addr: 0x{:X}", addr).unwrap();

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
            Err(err) => match err {
                I2CSlaveError::BufferOverflow => {
                    uwriteln!(&mut serial, "Error: {:?}", err).unwrap();
                    uwrite!(&mut serial, "Buffer content: ").unwrap();
                    buf.iter()
                        .for_each(|b| uwrite!(&mut serial, "{} ", b).unwrap());
                    uwrite!(&mut serial, "\n").unwrap();
                }
                I2CSlaveError::UnknownState((st, internal_state)) => uwriteln!(
                    &mut serial,
                    "Error: {:?}, 0x{:X}, {}",
                    err,
                    st,
                    internal_state
                )
                .unwrap(),
                I2CSlaveError::NotImplemented => {
                    uwriteln!(&mut serial, "Error: {:?}", err).unwrap()
                }
                I2CSlaveError::NotExpectedTransactionDirection => {
                    uwriteln!(&mut serial, "Error: {:?}", err).unwrap()
                }
            },
        };

        // RESPOND
        for b in buf.iter_mut() {
            *b *= 10;
        }

        match i2c_slave.respond(&mut buf) {
            Ok(count) => uwriteln!(&mut serial, "data has been sent: {}", count).unwrap(),
            Err(err) => uwriteln!(&mut serial, "{:?}", err).unwrap(),
        };
    }
}
