use esp_idf_sys::EspError;
use std::time::Duration;
use futures_timer::Delay;
use crate::spi::SpiTransportInterface;


pub struct Max7219<'a, T: SpiTransportInterface> {
    spi: &'a mut T,
}

pub async fn max7219_task<T>(mut spi: T) 
where
    T: SpiTransportInterface
{
    let mut this = Max7219::new(&mut spi);

    this.init().await.unwrap();
    this.run().await;
}

impl<'a, T: SpiTransportInterface> Max7219<'a, T> {

    pub fn new(spi: &'a mut T) -> Self {
        Self { spi }
    }

    pub async fn run(&mut self) {
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