#![warn(clippy::todo, clippy::unimplemented)]
use core::sync::atomic::{AtomicBool, Ordering};

use arduino_hal::{
    hal::port::{PC4, PC5},
    port::{
        mode::{Floating, Input},
        Pin,
    },
};
use avr_device::atmega328p::TWI;
use ufmt::{uDebug, uwrite};

// #[derive(uDebug, Clone)]
pub enum I2CSlaveError {
    BufferOverflow,
    UnknownState(u8), // Hex state
    NotImplemented,
    NotExpectedTransactionDirection,
}

impl uDebug for I2CSlaveError {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: arduino_hal::prelude::_ufmt_uWrite + ?Sized,
    {
        match self {
            I2CSlaveError::BufferOverflow => uwrite!(f, "BufferOverflow"),
            I2CSlaveError::UnknownState(state) => {
                uwrite!(f, "UnknownState: 0x{:X}", *state,)
            }
            I2CSlaveError::NotImplemented => uwrite!(f, "NotImplemented"),
            I2CSlaveError::NotExpectedTransactionDirection => {
                uwrite!(f, "NotExpectedTransactionDirection")
            }
        }
    }
}

#[allow(dead_code)]
pub struct I2cSlave<'a> {
    twi: TWI,
    addr: u8,
    sda: Pin<Input<Floating>, PC4>,
    scl: Pin<Input<Floating>, PC5>,
    int_flag: &'a AtomicBool,
}

impl<'a> I2cSlave<'a> {
    pub fn new(
        twi: TWI,
        addr: u8,
        sda: Pin<Input<Floating>, PC4>,
        scl: Pin<Input<Floating>, PC5>,
        int_flag: &'a AtomicBool,
    ) -> Self {
        Self {
            twi,
            addr,
            sda,
            scl,
            int_flag,
        }
    }

    /// Returns the init of this [`I2C_Slave`].
    pub fn init(&mut self, gca: bool) -> () {
        // Set slave address
        self.twi.twar.write(|w| w.twa().bits(self.addr));

        // Enable GCA call
        if gca {
            self.twi.twar.write(|w| w.twgce().set_bit());
        }

        self.twi.twcr.reset();

        ()
    }

    /// Set TWCR registers enabling TWI to respond [`I2cSlave`].
    fn arm(&self) -> () {
        // Arm TWI
        self.twi.twcr.write(|w| {
            w.twsta()
                .clear_bit()
                .twsto()
                .clear_bit()
                .twea()
                .set_bit()
                .twen()
                .set_bit()
                .twint()
                .set_bit()
                .twie()
                .set_bit()
        });
    }

    /// release moved values
    pub fn split(
        self,
    ) -> (
        TWI,
        Pin<Input<Floating>, PC4>,
        Pin<Input<Floating>, PC5>,
        &'a AtomicBool,
    ) {
        (self.twi, self.sda, self.scl, self.int_flag)
    }

    pub fn respond(&self, buffer: &[u8]) -> Result<usize, I2CSlaveError> {
        let mut i: usize = 0;
        let buffer_len: usize = buffer.len();
        let mut status: u8;

        self.arm();

        let result: Result<usize, I2CSlaveError>;

        // TODO loop may be reworked into something different
        result = loop {
            if self.int_flag.load(Ordering::SeqCst) {
                // Clearing prescaler bits according to datasheet to read
                // status codes correctly
                self.twi.twsr.write(|w| w.twps().bits(0));

                status = self.twi.twsr.read().bits();

                match status {
                    // Own SLA+W has been received; ACK has been returned, but we in read mode
                    0x60 => {
                        self.twi.twdr.write(|w| w.bits(0));

                        // Stop and disarm
                        self.twi
                            .twcr
                            .write(|w| w.twea().clear_bit().twint().set_bit());

                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::NotExpectedTransactionDirection);
                    }

                    // Own SLA+R has been received; ACK has been returned
                    0xA8 => {
                        if buffer_len == 0 {
                            // We have nothing to send
                            self.twi.twdr.write(|w| w.bits(0x00));
                            self.twi.twcr.write(|w| {
                                w.twint()
                                    .set_bit()
                                    .twea()
                                    .clear_bit()
                                    .twen()
                                    .set_bit()
                                    .twie()
                                    .set_bit()
                            });
                        } else {
                            // Send byte
                            self.twi.twdr.write(|w| w.bits(buffer[i]));

                            i += 1;

                            self.twi.twcr.write(|w| {
                                w.twint()
                                    .set_bit()
                                    .twea()
                                    .set_bit()
                                    .twen()
                                    .set_bit()
                                    .twie()
                                    .set_bit()
                            });
                        }

                        self.int_flag.store(false, Ordering::SeqCst);
                    }

                    // Arbitration lost in SLA+R/W as Master; own SLA+R has been
                    // received; ACK has been returned
                    0xB0 => {
                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::NotImplemented);
                    }
                    // Data byte in TWDR has been transmitted; ACK has been received
                    0xB8 => {
                        if i > buffer_len - 1 {
                            self.twi.twdr.write(|w| w.bits(0x00));

                            self.twi.twcr.write(|w| {
                                w.twint()
                                    .set_bit()
                                    .twea()
                                    .clear_bit()
                                    .twen()
                                    .set_bit()
                                    .twie()
                                    .set_bit()
                            });

                            break Ok(i);
                        } else {
                            self.twi.twdr.write(|w| w.bits(buffer[i]));

                            i += 1;

                            self.twi.twcr.write(|w| {
                                w.twint()
                                    .set_bit()
                                    .twea()
                                    .set_bit()
                                    .twen()
                                    .set_bit()
                                    .twie()
                                    .set_bit()
                            });
                        }

                        self.int_flag.store(false, Ordering::SeqCst);
                    }
                    // Data byte in TWDR has been transmitted; NOT ACK has been received
                    0xC0 => {
                        self.twi
                            .twcr
                            .write(|w| w.twint().set_bit().twea().clear_bit());

                        self.int_flag.store(false, Ordering::SeqCst);

                        break Ok(i);
                    }
                    // Last data byte in TWDR has been transmitted (TWEA = “0”);
                    // ACK has been received
                    0xC8 => {
                        self.twi
                            .twcr
                            .write(|w| w.twint().set_bit().twea().clear_bit());

                        self.int_flag.store(false, Ordering::SeqCst);

                        break Ok(i);
                    }
                    0xF8 => {
                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        // ERROR
                        break Err(I2CSlaveError::UnknownState(status));
                    }
                    _ => {
                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::UnknownState(status));
                    }
                }
            }
        };

        self.twi.twcr.reset();
        result
    }

    /// Receive data and write it to buffer
    pub fn receive(&self, buffer: &mut [u8]) -> Result<(), I2CSlaveError> {
        let mut i: usize = 0;
        let buffer_len: usize = buffer.len();
        let mut status: u8;
        let mut internal_state: u8 = 1;

        self.arm();

        let result: Result<(), I2CSlaveError>;

        // Read I2C in blocking mode
        result = loop {
            if self.int_flag.load(Ordering::SeqCst) {
                // Clearing prescaler bits according to datasheet to read
                // status codes correctly
                self.twi.twsr.write(|w| w.twps().bits(0));

                status = self.twi.twsr.read().bits();

                match status {
                    // READ mode is not expected
                    0xA8 => {
                        self.twi.twdr.write(|w| w.bits(0));

                        // Stop and disarm
                        self.twi
                            .twcr
                            .write(|w| w.twea().clear_bit().twint().set_bit());

                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::NotExpectedTransactionDirection);
                    }

                    // Own SLA+W has been received; ACK has been returned
                    0x60 => {
                        // Continue, wait for data
                        self.twi.twcr.write(|w| {
                            w.twint()
                                .set_bit()
                                .twea()
                                .set_bit()
                                .twie()
                                .set_bit()
                                .twen()
                                .set_bit()
                        });
                        internal_state = internal_state << 1;

                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);
                    }

                    // Arbitration lost in SLA+R/W as Master; own SLA+W has been
                    // received; ACK has been returned
                    0x68 => {
                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::NotImplemented);
                    }

                    // General call address has been received; ACK has been returned
                    0x70 => {
                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::NotImplemented);
                    }

                    // Arbitration lost in SLA+R/W as Master; General call
                    // address has been received; ACK has been returned
                    0x78 => {
                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::NotImplemented);
                    }

                    // Previously addressed with own SLA+W; data has been received;
                    // ACK has been returned
                    0x80 => {
                        internal_state = internal_state << 1;

                        if i > buffer_len - 1 {
                            // Stop and disarm
                            self.twi
                                .twcr
                                .write(|w| w.twea().clear_bit().twint().set_bit());

                            // Resetting flag
                            self.int_flag.store(false, Ordering::SeqCst);

                            break Err(I2CSlaveError::BufferOverflow);
                        } else {
                            // Write data to buffer
                            buffer[i] = self.twi.twdr.read().bits();

                            i += 1;

                            // Wait for more
                            self.twi.twcr.write(|w| {
                                w.twea()
                                    .set_bit()
                                    .twen()
                                    .set_bit()
                                    .twint()
                                    .set_bit()
                                    .twie()
                                    .set_bit()
                            });

                            // Resetting flag
                            self.int_flag.store(false, Ordering::SeqCst);
                        }
                    }
                    // Previously addressed with general call; data has been
                    // received; ACK has been returned
                    0x90 => {
                        // TODO GCA mode
                        buffer[i] = self.twi.twdr.read().bits();

                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::NotImplemented);
                    }
                    // A STOP condition or repeated START condition has been
                    // received while still addressed as Slave
                    0xA0 => {
                        // Stop and disarm
                        self.twi
                            .twcr
                            .write(|w| w.twea().clear_bit().twint().set_bit());

                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Ok(());
                    }
                    0xf8 => {
                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        // ERROR
                        break Err(I2CSlaveError::UnknownState(status));
                    }
                    _ => {
                        // Resetting flag
                        self.int_flag.store(false, Ordering::SeqCst);

                        break Err(I2CSlaveError::UnknownState(status));
                    }
                }
            }
        };

        self.twi.twcr.reset();

        result
    }
}

#[cfg(test)]
mod tests {

    fn slice_size(b: &mut [u8]) -> usize {
        b.len()
    }

    #[test]
    fn sample_tests() {
        let mut buffer: [u8; 4] = [0; 4];

        assert_eq!(slice_size(&buffer), 4);
    }
}
