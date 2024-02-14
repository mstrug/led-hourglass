use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::*;
use esp_idf_sys::EspError;
use esp_idf_svc::hal::gpio;
use esp_idf_hal::spi::config::*;
use crate::global_config::*;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::gpio::OutputPin;

pub trait SpiWrite {
    async fn write(&mut self, data: &[u8]) -> Result<(), EspError>;
}

pub struct SpiInterface<'a> {
    spi: SpiDeviceDriver<'a, SpiDriver<'a>>,
}

impl<'a> SpiInterface<'a> {

//    pub fn init(global_config: GlobalConfig) -> Result<Self, EspError> {
  pub fn init(spia: impl Peripheral<P = impl SpiAnyPins> + 'a, gpio_mosi: impl Peripheral<P = impl OutputPin> + 'a, gpio_clk: impl Peripheral<P = impl OutputPin> + 'a, gpio_cs: impl Peripheral<P = impl OutputPin> + 'a) -> Result<Self, EspError> {

    // let gpio_mosi = global_config.led(); //global_config_gpio(ConfigSystemFeatures::SpiMosi);
    // let gpio_clk = global_config.led(); //global_config_gpio(ConfigSystemFeatures::SpiClk);
    // let gpio_cs = global_config.led(); //global_config_gpio(ConfigSystemFeatures::SpiCs);

    //let peripherals = Peripherals::take()?;

        let spi_drv = SpiDriver::new(
            //peripherals.spi2,
            spia,
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

impl<'a> SpiWrite for SpiInterface<'a> {

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