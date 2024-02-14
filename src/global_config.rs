use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_svc::hal::gpio;
use esp_idf_hal::gpio::AnyOutputPin;


pub enum ConfigSystemFeatures {
    LedHeartbeatGpio,
    SpiMosi,
    SpiClk,
    SpiCs,
}

pub struct GlobalConfig {
    peripherals: Peripherals
}

impl GlobalConfig {

    pub fn new() -> Self {
        Self {
            peripherals: Peripherals::take().unwrap()
        }
    }

    // pub fn led(&self) -> gpio::Gpio5 {
    //     self.peripherals.pins.gpio5
    // }
    

    // pub fn global_config_gpio(&self, feature: ConfigSystemFeatures) -> impl OutputPin {

    //     match feature {
    //         ConfigSystemFeatures::LedHeartbeatGpio => self.peripherals.pins.gpio5.downgrade_output(),
    //         ConfigSystemFeatures::SpiMosi => self.peripherals.pins.gpio18.downgrade_output(),
    //         ConfigSystemFeatures::SpiClk => self.peripherals.pins.gpio23.downgrade_output(),
    //         ConfigSystemFeatures::SpiCs => self.peripherals.pins.gpio17.downgrade_output(),
    //     }
    // }

}
