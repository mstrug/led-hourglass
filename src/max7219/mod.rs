use esp_idf_sys::EspError;
use std::time::Duration;
use futures_timer::Delay;
use crate::spi::SpiTransportInterface;
use async_channel::Receiver;


pub enum Max7219Action {
    ClearScreen,
    SetLedState { x: u8, y: u8, on: bool },
}


pub struct Max7219<'a, T: SpiTransportInterface> {
    spi: &'a mut T,
    led_states: [u8;8],
    update: bool,
    client: Option<Receiver<Max7219Action>>,
}

pub async fn max7219_task<T>(mut spi: T, client: Option<Receiver<Max7219Action>>) 
where
    T: SpiTransportInterface
{
    let mut this = Max7219::new(&mut spi, client);

    this.init().await.unwrap();
    this.run().await;
}

impl<'a, T: SpiTransportInterface> Max7219<'a, T> {

    pub fn new(spi: &'a mut T, client: Option<Receiver<Max7219Action>>) -> Self {
        Self { spi,
            led_states: [0;8],
            update: true,
            client,
        }
    }

    pub fn set_led(&mut self, x: u8, y: u8, on: bool) {
        if x >= 8 || y >= 8 {
            return;
        } 

        let y = y as usize;
        let x = 1 << x;

        if on && ((self.led_states[y] & x) == 0) {
            self.led_states[y] |= x;
            self.update = true;
        } else if !on && ((self.led_states[y] & x) == x) {
            self.led_states[y] &= !x;
            self.update = true;
        }
    }

    pub async fn run(&mut self) {
        log::info!("Max7219 started");

        self.set_led(1, 1, true);
        self.set_led(1, 7, true);

        loop {


            if self.update {
                for addr in 0..8 {
                    let send_array: [u8; 2] = [addr + 1, self.led_states[addr as usize]];
                    self.spi.write(&send_array).await.unwrap();
                }
                self.update = false;
            } else {
                // self.set_led(4, 4, true);
                // self.set_led(1, 1, false);
                // async_std::task::yield_now().await;
            }


            if let Some(client) = &self.client {
                let input_command = client.recv().await.unwrap();

                match input_command {
                    Max7219Action::ClearScreen => {
                            for i in 0..8 { self.led_states[i] = 0; }
                            self.update = true;
                        }
                    Max7219Action::SetLedState { x, y, on } => {
                        self.set_led(x, y, on);
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub async fn run_demo(&mut self) {
        log::info!("Max7219 started");

        loop {
            let mut data: u8 = 1;
            // Iterate over all rows of LED matrix
            for addr in 1..9 {
                // addr refrences the row data will be sent to
                let send_array: [u8; 2] = [addr, data];
                // Shift a 1 with evey loop
                data = data << 1;

                // Send data just like earlier
                self.spi.write(&send_array).await.unwrap();

                // Delay for 500ms to show effect
                Delay::new(Duration::from_millis(40)).await;
            }

            // Clear the LED matrix row by row with 500ms delay in between
            for addr in 1..9 {
                let send_array: [u8; 2] = [addr, data];
                self.spi.write(&send_array).await.unwrap();
                Delay::new(Duration::from_millis(40)).await;
            }
        }
    }

    pub async fn init(&mut self) -> Result<(), EspError> {
        self.spi.write(&[0x0C, 0x00]).await?;  // power off
        self.spi.write(&[0x0F, 0x00]).await?;  // disable test mode
        self.spi.write(&[0x0A, 0x00]).await?;  // Intensity low
        self.spi.write(&[0x09, 0x00]).await?;  // Set up Decode Mode
        self.spi.write(&[0x0B, 0x07]).await?;    // Configure Scan Limit

        // Clear the LED matrix row by row with 500ms delay in between
        for addr in 1..9 {
            self.spi.write(&[addr, 0]).await?;
        }
        self.spi.write(&[0x0C, 0x01]).await?;  // power on

        log::info!("Max7219 init done");

        Ok(())
    }
}