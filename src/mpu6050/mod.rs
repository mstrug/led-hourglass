use esp_idf_hal::i2c::I2cDriver;
use mpu6050_dmp::address::Address;
use std::time::Duration;
use esp_idf_sys::EspError;
use crate::i2c::I2cTransportInterface;
use std::time::SystemTime;
use async_channel::Sender;


const DEFAULT_ADDRESS: u8 = 0x68;

#[derive(PartialEq, Default)]
pub struct Mpu6050ObserverData {
    pub acc_vec: (f32, f32, f32),
    pub acc_angle: (f32, f32, f32),
}

pub struct Mpu6050<'a, T: I2cTransportInterface> {
    i2c: &'a mut T,
    temperature: f32,
    acc_vec: (f32, f32, f32),
    acc_err: (f32, f32, f32),
    acc_angle: (f32, f32, f32),
    gyro_vec: (f32, f32, f32),
    gyro_err: (f32, f32, f32),
    gyro_angle: (f32, f32, f32),
    read_time_prev: SystemTime,
    observer: Option<Sender<Mpu6050ObserverData>>
}

pub async fn mpu6050_task<T>(mut i2c: T, observer: Option<Sender<Mpu6050ObserverData>>) 
where
    T: I2cTransportInterface
{
    let mut this = Mpu6050::new(&mut i2c, observer);

    this.init().await.unwrap();
    this.run().await;
}

impl<'a, T: I2cTransportInterface> Mpu6050<'a, T> {

    pub fn new(i2c: &'a mut T, observer: Option<Sender<Mpu6050ObserverData>>) -> Self {
        Self { i2c,
            temperature: 0f32,
            acc_vec: (0f32,0f32,0f32),
            acc_err: (0f32,0f32,0f32),
            acc_angle: (0f32,0f32,0f32),
            gyro_vec: (0f32,0f32,0f32),
            gyro_err: (0f32,0f32,0f32),
            gyro_angle: (0f32,0f32,0f32),
            read_time_prev: SystemTime::now(),
            observer
         }
    }

    pub async fn init(&mut self) -> Result<(), EspError> {
        self.i2c.write(DEFAULT_ADDRESS, &[0x6B, 0x00]).await?; // reset 

        // config register 0x1C
        //self.i2c.write(DEFAULT_ADDRESS, &[0x1C, 0x10]).await?; // Set the register bits as 00010000 (+/- 8g full scale range) 
        //self.i2c.write(DEFAULT_ADDRESS, &[0x1B, 0x10]).await?; // Set the register bits as 00010000 (1000deg/s full scale)

        self.calculate_error().await;

        log::info!("Mpu6050 init done");

        Ok(())
    }

    async fn calculate_error(&mut self) {
        let max_iter = 200;
        let mut sum_x = 0f32;
        let mut sum_y = 0f32;
        let mut sum_z = 0f32;

        for _ in 0..max_iter {
            self.read_accelerometer().await;
            sum_x += self.acc_vec.0;
            sum_y += self.acc_vec.1;
            sum_z += self.acc_vec.2;
        }
        // assuming device lays on flat surface, x and y acceleration vectors should be 0 and z vector should be 1 (g)
        self.acc_err.0 = 0f32 - sum_x / max_iter as f32;
        self.acc_err.1 = 0f32 - sum_y / max_iter as f32;
        self.acc_err.2 = 1f32 - sum_z / max_iter as f32;

        sum_x = 0f32;
        sum_y = 0f32;
        sum_z = 0f32;
        for _ in 0..max_iter {
            self.read_gyroscope().await;
            sum_x += self.gyro_vec.0;
            sum_y += self.gyro_vec.1;
            sum_z += self.gyro_vec.2;
        }
        self.gyro_err.0 = sum_x / max_iter as f32;
        self.gyro_err.1 = sum_y / max_iter as f32;
        self.gyro_err.2 = sum_z / max_iter as f32;

        println!("Accelerometer error: x={}, y={}, z={}", self.acc_err.0, self.acc_err.1, self.acc_err.2);
        println!("Gyroscope error: x={}, y={}, z={}", self.gyro_err.0, self.gyro_err.1, self.gyro_err.2);
        futures_timer::Delay::new(Duration::from_millis(2000)).await;
    }

    async fn read_accelerometer(&mut self) {
        let mut buf = [0u8; 6];
        self.i2c.write_read(DEFAULT_ADDRESS, &[0x3B], &mut buf).await.unwrap(); // Start with register 0x3B (ACCEL_XOUT_H)
        //For a range of +-2g, we need to divide the raw values by 16384, according to the datasheet
        self.acc_vec.0 = ((buf[0] as i16) << 8 | (buf[1] as i16)) as f32 / 16384_f32; // x
        self.acc_vec.1 = ((buf[2] as i16) << 8 | (buf[3] as i16)) as f32 / 16384_f32; // y
        self.acc_vec.2 = ((buf[4] as i16) << 8 | (buf[5] as i16)) as f32 / 16384_f32; // z

        self.acc_vec.0 += self.acc_err.0;
        self.acc_vec.1 += self.acc_err.1;
        self.acc_vec.2 += self.acc_err.2;
    }

    async fn read_gyroscope(&mut self) {
        let mut buf = [0u8; 6];
        self.i2c.write_read(DEFAULT_ADDRESS, &[0x43], &mut buf).await.unwrap(); // Gyro data first register address 0x43
        // For a 250deg/s range we have to divide first the raw value by 131.0, according to the datasheet
        self.gyro_vec.0 = ((buf[0] as i16) << 8 | (buf[1] as i16)) as f32 / 131_f32; // x
        self.gyro_vec.1 = ((buf[2] as i16) << 8 | (buf[3] as i16)) as f32 / 131_f32; // y
        self.gyro_vec.2 = ((buf[4] as i16) << 8 | (buf[5] as i16)) as f32 / 131_f32; // z
    }

    async fn read_temperature(&mut self) {
        let mut buf = [0u8; 2];
        self.i2c.write_read(DEFAULT_ADDRESS, &[0x41], &mut buf).await.unwrap(); // Temp data first register address 0x41
        self.temperature = ((buf[0] as i16) << 8 | (buf[1] as i16)) as f32 / 340_f32 + 36.53_f32;
    }

    pub async fn run(&mut self) {
        log::info!("Mpu6050 started");

        let mut print_time = SystemTime::now();
        let mut old_data = Mpu6050ObserverData { 
            acc_vec: self.acc_vec, 
            acc_angle: self.acc_angle,
        };

        loop {
            self.read_temperature().await;

            self.read_accelerometer().await;

            let digi_places = 1;

            self.acc_vec.0 = round(self.acc_vec.0, digi_places);
            self.acc_vec.1 = round(self.acc_vec.1, digi_places);
            self.acc_vec.2 = round(self.acc_vec.2, digi_places);

            if self.acc_vec.2 == 0f32 {
                self.acc_vec.2 = 0.00000001f32;
            }
            if self.acc_vec.0 == self.acc_vec.2 {
                self.acc_vec.0 += 0.00000001f32;
            }
            if self.acc_vec.1 == self.acc_vec.2 {
                self.acc_vec.1 += 0.00000001f32;
            }

            let acc_angle_x: f32 = round( (self.acc_vec.1 / (self.acc_vec.0.powf(2f32) + self.acc_vec.2.powf(2f32)).sqrt()).atan() * 180f32 / std::f32::consts::PI, digi_places);
            let acc_angle_y: f32 = round( (self.acc_vec.0 / (self.acc_vec.1.powf(2f32) + self.acc_vec.2.powf(2f32)).sqrt()).atan() * 180f32 / std::f32::consts::PI, digi_places);
            let acc_angle_z: f32 = round( ((self.acc_vec.0.powf(2f32) + self.acc_vec.1.powf(2f32)).sqrt() / self.acc_vec.2 ).atan() * 180f32 / std::f32::consts::PI, digi_places);

            self.acc_angle.0 = acc_angle_x;
            self.acc_angle.1 = acc_angle_y;
            self.acc_angle.2 = acc_angle_z;

            let current_time = SystemTime::now();
            let delta_time = current_time.duration_since(self.read_time_prev).unwrap().as_secs_f32();
            self.read_time_prev = current_time;

            self.read_gyroscope().await;

            self.gyro_vec.0 -= self.gyro_err.0;
            self.gyro_vec.1 -= self.gyro_err.1;
            self.gyro_vec.2 -= self.gyro_err.2;

            self.gyro_angle.0 += self.gyro_vec.0 * delta_time; // deg/s * s = deg
            self.gyro_angle.1 += self.gyro_vec.1 * delta_time;
            self.gyro_angle.2 += self.gyro_vec.2 * delta_time;

            let _roll: f32 = 0.96f32 * self.gyro_angle.0 + 0.04f32 * acc_angle_x;
            let _pitch: f32 = 0.96f32 * self.gyro_angle.1 + 0.04f32 * acc_angle_y;
            let _yaw: f32 = self.gyro_angle.2;

            if current_time.duration_since(print_time).unwrap().as_millis() > 500 {
                print_time = current_time;
                //log::info!("temperature:       {}", self.temperature);
                //log::info!("accelerometer:   v = ( {:1.1} , {:1.1} , {:1.1} )   ang = ( {:1.1} , {:1.1} , {:1.1} )", self.acc_vec.0, self.acc_vec.1, self.acc_vec.2, acc_angle_x, acc_angle_y, acc_angle_z);
                //log::info!("gyroscope:       ( {} , {} , {} )", self.gyro_vec.0, self.gyro_vec.1, self.gyro_vec.2);
                //log::info!("gyroscope angle: ( {} , {} , {} )", self.gyro_angle.0, self.gyro_angle.1, self.gyro_angle.2);
                //log::info!("roll/pitch/yaw:  ( {} , {} , {} )\n", roll, pitch, yaw);
            }

            if let Some(observer) = &self.observer {
                let new_data = Mpu6050ObserverData { 
                    acc_vec: self.acc_vec, 
                    acc_angle: self.acc_angle,
                };
                if old_data != new_data { // todo: compare only up to 0.1
                    old_data.acc_vec = new_data.acc_vec;
                    old_data.acc_angle = new_data.acc_angle;
                    observer.send(new_data).await.unwrap();
                }
            }
            futures_timer::Delay::new(Duration::from_millis(10)).await;
        }

    }
}


#[allow(dead_code)]
pub async fn mpu6050_task1(i2c: I2cDriver<'_>) 
{
    // let mut mpu = Mpu6050::new(i2c);
    //let mut delay = esp_idf_hal::delay::Delay::new_default() as 
    // let mut delay = embedded_hal::delay::DelayMs;
    // mpu.init(&mut delay).unwrap();

    let mut sensor = mpu6050_dmp::sensor::Mpu6050::new(i2c, Address::default()).unwrap();

    let mut delay = esp_idf_hal::delay::Delay::new_default();
    sensor.initialize_dmp(&mut delay).unwrap();
    log::info!("Max7219 init done");

    log::info!("Max7219 started");

    loop {
        if let Ok(acc) = sensor.accel() {
            log::info!("ACC({},{},{})", acc.x(), acc.y(), acc.z());
        }
        if let Ok(gy) = sensor.gyro() {
            log::info!("GYRO({},{},{})", gy.x(), gy.y(), gy.z());
        }
        futures_timer::Delay::new(Duration::from_millis(500)).await;
    }
}

fn round(x: f32, decimals: u32) -> f32 {
    let y = 10i32.pow(decimals) as f32;
    (x * y).round() / y
}
