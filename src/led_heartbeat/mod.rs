use crate::global_config::*;
use esp_idf_hal::gpio::*;
use futures::Future;
use std::time::Duration;
use futures_timer::Delay;



pub async fn led_heartbeat_task<'a>(mut led: PinDriver<'a, impl OutputPin, Output>)
{
    log::info!("LED HeartBeat started");

    loop {
        led.set_high().ok();
        Delay::new(Duration::from_millis(500)).await;
        led.set_low().ok();
        Delay::new(Duration::from_millis(500)).await;
    }
}
