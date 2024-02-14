use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use edge_executor::{Executor, LocalExecutor};
//use async_channel::{unbounded, Receiver, Sender};

#[macro_use]
extern crate lazy_static;

mod global_config;
use global_config::*;
mod led_heartbeat;
use led_heartbeat::*;
mod max7219;
use max7219::*;
mod spi;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_svc::hal::gpio;
use esp_idf_hal::gpio::AnyOutputPin;


async fn app<'a>(rt: &Executor<'a>) {
    log::info!("App started");

    let peripherals = Peripherals::take().unwrap();
    let led = PinDriver::output(peripherals.pins.gpio5).unwrap();

    let task1 = rt.spawn(led_heartbeat_task(led));

    let mosi = peripherals.pins.gpio23;
    let clk = peripherals.pins.gpio18;
    let cs = peripherals.pins.gpio17;
    let spi = spi::SpiInterface::init(peripherals.spi2, mosi, clk, cs).unwrap();

    let task2 = rt.spawn(max7219_task(spi));

    futures::join!(task1, task2);

    log::info!("App end");
}


fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Started");

    //    let ex: LocalExecutor = LocalExecutor::new();
    let ex: Executor = Executor::new();
    let future = app(&ex);
    edge_executor::block_on(ex.run(future));
}
