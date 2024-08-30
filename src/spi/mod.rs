use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::*;
use esp_idf_sys::EspError;
use esp_idf_svc::hal::gpio;
use esp_idf_hal::spi::config::*;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::gpio::AnyIOPin;


pub trait SpiTransportInterface {
    async fn write(&mut self, _data: &[u8]) -> Result<(), EspError> { Ok(()) }
    async fn read(&mut self, _data: &[u8]) -> Result<(), EspError> { Ok(()) }
}

pub struct SpiInterface<'a> {
    spi: SpiDeviceDriver<'a, SpiDriver<'a>>,
}

impl<'a> SpiInterface<'a> {

  pub fn init(spi: impl Peripheral<P = impl SpiAnyPins> + 'a, gpio_mosi: AnyIOPin, gpio_clk: AnyIOPin, gpio_cs: AnyIOPin) -> Result<Self, EspError> {

        let spi_drv = SpiDriver::new(
            spi,
            gpio_clk,
            gpio_mosi,
            None::<gpio::AnyIOPin>,
            &SpiDriverConfig::new(),
        )?;

        let config = Config::new().baudrate(2.MHz().into()).data_mode(Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        });

        log::info!("SPI started");

        Ok(Self {
            spi: SpiDeviceDriver::new(spi_drv, Some(gpio_cs), &config)?,
        })
    }
}

impl<'a> SpiTransportInterface for SpiInterface<'a> {

    async fn write(&mut self, data: &[u8]) -> Result<(), EspError> {
        // todo self.spi.write_async(&data).await;
        self.spi.write(&data)
    }

    // todo: change to macro
    // pub fn write(&mut self, address: u8, data: &[u8]) -> Result<(), EspError> {
    //     assert!(self.buffer.len() < data.len(), "Increase SPI internal buffer");
    //     self.buffer[0] = address;
    //     data.iter().enumerate().for_each(|(idx,i)| self.buffer[idx + 1] = *i);
    //     self.spi.write(&self.buffer)
    // }

}