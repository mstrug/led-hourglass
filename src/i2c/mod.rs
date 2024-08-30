use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::i2c::{I2c, I2cConfig, I2cDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::*;
use esp_idf_sys::EspError;
use esp_idf_svc::sys::TickType_t;
use esp_idf_hal::delay::BLOCK;
use std::num::NonZeroI32;



pub trait I2cTransportInterface {

    // read up to output len
    async fn write_read(&mut self, address: u8, data_to_write: &[u8], output: &mut [u8]) -> Result<(), EspError>;

    async fn write(&mut self, address: u8, data: &[u8]) -> Result<(), EspError>;

    // read up to output len
    async fn read(&mut self, address: u8, output: &mut [u8]) -> Result<(), EspError>;
}


pub struct I2cInterface<'a> {
    i2c: I2cDriver<'a>,
    //timeout: TickType_t,
}

impl<'a> I2cInterface<'a> {

  pub fn init(i2c: impl Peripheral<P = impl I2c> + 'a, gpio_sda: AnyIOPin, gpio_scl: AnyIOPin) -> Result<Self, EspError> {

        let config = I2cConfig::new().baudrate(100.kHz().into());

        log::info!("I2C started");

        Ok(Self {
            i2c: I2cDriver::new(i2c, gpio_sda, gpio_scl, &config)?,
        })
    }
}

impl<'a> I2cTransportInterface for I2cInterface<'a> {

    async fn write_read(&mut self, address: u8, data_to_write: &[u8], output: &mut [u8]) -> Result<(), EspError> {
        self.i2c.write_read(address, &data_to_write, output, BLOCK)
    }

    async fn write(&mut self, address: u8, data_to_write: &[u8]) -> Result<(), EspError> {
        self.i2c.write(address, &data_to_write, BLOCK)
    }

    async fn read(&mut self, address: u8, output: &mut [u8]) -> Result<(), EspError> {
        self.i2c.read(address, output, BLOCK)
    }



    // todo: change to macro
    // pub fn write(&mut self, address: u8, data: &[u8]) -> Result<(), EspError> {
    //     assert!(self.buffer.len() < data.len(), "Increase SPI internal buffer");
    //     self.buffer[0] = address;
    //     data.iter().enumerate().for_each(|(idx,i)| self.buffer[idx + 1] = *i);
    //     self.spi.write(&self.buffer)
    // }

}


