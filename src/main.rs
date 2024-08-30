use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use edge_executor::Executor;
//use async_channel::{unbounded, Receiver, Sender};
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;

mod led_heartbeat;
use led_heartbeat::*;
mod max7219;
use max7219::*;
mod mpu6050;
use mpu6050::*;
mod spi;
mod i2c;



async fn app<'a>(rt: &Executor<'a>) {
    log::info!("App started");

    // Take Peripherals object (possible only once)
    let peripherals = Peripherals::take().unwrap();

    // Setup SPI 
    let mosi = peripherals.pins.gpio23;
    let clk = peripherals.pins.gpio18;
    let cs = peripherals.pins.gpio17;
    let spi = peripherals.spi2;
    let spi_interface = spi::SpiInterface::init(spi, mosi.into(), clk.into(), cs.into()).unwrap();

    // Setup I2C
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let i2c = peripherals.i2c0;
    let i2c_master = i2c::I2cInterface::init(i2c, sda.into(), scl.into()).unwrap();


    // Setup led heartbeat task 
    let led = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let task1 = rt.spawn(led_heartbeat_task(led));

    // Setup max7219 task 
    let task2 = rt.spawn(max7219_task(spi_interface));

    // Setup mpu6050 task
    let task3 = rt.spawn(mpu6050_task(i2c_master));

    // Start all task and wait until finished
    futures::join!(task1, task2, task3);

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
