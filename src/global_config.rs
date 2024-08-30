use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::spi::SpiAnyPins;
use esp_idf_svc::hal::gpio;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_svc::hal::peripheral::Peripheral;

pub enum ConfigSystemFeatures {
    LedHeartbeatGpio,
    SpiMosi,
    SpiClk,
    SpiCs,
}


// pub fn global_config_gpio(peripherals: &Peripherals, feature: ConfigSystemFeatures) -> &impl OutputPin {
//     match feature {
//         ConfigSystemFeatures::LedHeartbeatGpio => &peripherals.pins.gpio5.downgrade_output(),
//         ConfigSystemFeatures::SpiMosi => &peripherals.pins.gpio23.downgrade_output(),
//         ConfigSystemFeatures::SpiClk => &peripherals.pins.gpio18.downgrade_output(),
//         ConfigSystemFeatures::SpiCs => &peripherals.pins.gpio17.downgrade_output(),
//     }
// }

// pub fn global_config_spi(peripherals: &Peripherals) -> &impl Peripheral<P = impl SpiAnyPins> {
//     &peripherals.spi2
// }
