use esp_idf_hal::i2c::I2cDriver;
use mpu6050_dmp::sensor::*;
use mpu6050_dmp::address::Address;
use std::ops::Neg;
use std::time::Duration;
use esp_idf_hal::prelude::*;
use esp_idf_hal::i2c::*;
use esp_idf_sys::EspError;
use esp_idf_svc::hal::gpio;
use esp_idf_hal::spi::config::*;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::gpio::OutputPin;
use crate::i2c::I2cTransportInterface;
use std::time::SystemTime;


const DEFAULT_ADDRESS: u8 = 0x68;

pub struct Mpu6050<'a, T: I2cTransportInterface> {
    i2c: &'a mut T,
    temperature: f32,
    acc_vec: (f32, f32, f32),
    acc_err: (f32, f32, f32),
    gyro_vec: (f32, f32, f32),
    gyro_err: (f32, f32, f32),
    gyro_angle: (f32, f32, f32),
    read_time_prev: SystemTime,
}

pub async fn mpu6050_task<T>(mut i2c: T) 
where
    T: I2cTransportInterface
{
    let mut this = Mpu6050::new(&mut i2c);

    this.init().await.unwrap();
    this.run().await;
}

impl<'a, T: I2cTransportInterface> Mpu6050<'a, T> {

    pub fn new(i2c: &'a mut T) -> Self {
        Self { i2c,
            temperature: 0f32,
            acc_vec: (0f32,0f32,0f32),
            acc_err: (0f32,0f32,0f32),
            gyro_vec: (0f32,0f32,0f32),
            gyro_err: (0f32,0f32,0f32),
            gyro_angle: (0f32,0f32,0f32),
            read_time_prev: SystemTime::now(),
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

        for _ in 0..max_iter {
            let (acc_angle_x, acc_angle_y) = self.read_accelerometer().await;
            sum_x += acc_angle_x;
            sum_y += acc_angle_y;
        }
        self.acc_err.0 = sum_x / max_iter as f32;
        self.acc_err.1 = sum_y / max_iter as f32;

        sum_x = 0f32;
        sum_y = 0f32;
        let mut sum_z = 0f32;
        for _ in 0..max_iter {
            self.read_gyroscope().await;
            sum_x += self.gyro_vec.0;
            sum_y += self.gyro_vec.1;
            sum_z += self.gyro_vec.2;
        }
        self.gyro_err.0 = sum_x / max_iter as f32;
        self.gyro_err.1 = sum_y / max_iter as f32;
        self.gyro_err.2 = sum_z / max_iter as f32;

        println!("Accelerometer error: x={}, y={}", self.acc_err.0, self.acc_err.1);
        println!("Gyroscope error: x={}, y={}, z={}", self.gyro_err.0, self.gyro_err.1, self.gyro_err.2);
        futures_timer::Delay::new(Duration::from_millis(2000)).await;
    }

    async fn read_accelerometer(&mut self) -> (f32, f32) {
        let mut buf = [0u8; 6];
        self.i2c.write_read(DEFAULT_ADDRESS, &[0x3B], &mut buf).await.unwrap(); // Start with register 0x3B (ACCEL_XOUT_H)
        //For a range of +-2g, we need to divide the raw values by 16384, according to the datasheet
        self.acc_vec.0 = ((buf[0] as i16) << 8 | (buf[1] as i16)) as f32 / 16384_f32; // x
        self.acc_vec.1 = ((buf[2] as i16) << 8 | (buf[3] as i16)) as f32 / 16384_f32; // y
        self.acc_vec.2 = ((buf[4] as i16) << 8 | (buf[5] as i16)) as f32 / 16384_f32; // z

        let acc_angle_x: f32 = ((self.acc_vec.1 / (self.acc_vec.0.powf(2f32) + self.acc_vec.2.powf(2f32)).sqrt()).atan() * 180f32 / std::f32::consts::PI) - self.acc_err.0;
        let acc_angle_y: f32 = ((self.acc_vec.0.neg() / (self.acc_vec.1.powf(2f32) + self.acc_vec.2.powf(2f32)).sqrt()).atan() * 180f32 / std::f32::consts::PI) - self.acc_err.1;
        (acc_angle_x, acc_angle_y)
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

        loop {
            self.read_temperature().await;

            let (acc_angle_x, acc_angle_y) = self.read_accelerometer().await;

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

            let roll: f32 = 0.96f32 * self.gyro_angle.0 + 0.04f32 * acc_angle_x;
            let pitch: f32 = 0.96f32 * self.gyro_angle.1 + 0.04f32 * acc_angle_y;
            let yaw: f32 = self.gyro_angle.2;

            if current_time.duration_since(print_time).unwrap().as_millis() > 500 {
                print_time = current_time;
                log::info!("temperature:       {}", self.temperature);
                log::info!("accelerometer:   ( {} , {} , {} )", self.acc_vec.0, self.acc_vec.1, self.acc_vec.2);
                log::info!("accel. angle:    ( {} , {} )", acc_angle_x, acc_angle_y);
                log::info!("gyroscope:       ( {} , {} , {} )", self.gyro_vec.0, self.gyro_vec.1, self.gyro_vec.2);
                log::info!("gyroscope angle: ( {} , {} , {} )", self.gyro_angle.0, self.gyro_angle.1, self.gyro_angle.2);
                log::info!("roll/pitch/yaw:  ( {} , {} , {} )\n", roll, pitch, yaw);
                //futures_timer::Delay::new(Duration::from_millis(500)).await;
            }
            futures_timer::Delay::new(Duration::from_millis(10)).await;
        }

    }
}


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


