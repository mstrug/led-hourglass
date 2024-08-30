use std::time::SystemTime;
use async_channel::{Receiver, Sender};
use super::mpu6050::Mpu6050ObserverData;
use super::max7219::Max7219Action;


pub struct Logic {
    acc_observer: Receiver<Mpu6050ObserverData>,
    led_matrix_server: Sender<Max7219Action>,

    pos: (u8, u8),
    old_pos: (u8, u8),
}

pub async fn logic_task(acc_observer: Receiver<Mpu6050ObserverData>, led_matrix_server: Sender<Max7219Action>)
{
    let mut this = Logic {
        acc_observer,
        led_matrix_server,
        pos: (3, 3),
        old_pos: (0, 0),
    };

    this.run().await;
}

impl Logic {

    async fn update_led_matrix(&self) {
        self.led_matrix_server.send(Max7219Action::SetLedState { x: self.old_pos.0, y: self.old_pos.1, on: false }).await.unwrap();
        self.led_matrix_server.send(Max7219Action::SetLedState { x: self.pos.0, y: self.pos.1, on: true }).await.unwrap();
    }

    async fn clear_led_matrix(&self) {
        self.led_matrix_server.send(Max7219Action::ClearScreen).await.unwrap();
    }

    #[allow(dead_code)]
    fn handle_logic_acc_vec(&mut self, diff_x: f32, diff_y: f32) {
        let max_x = 8;
        let min_x = 0;
        let max_y = 8;
        let min_y = 0;
        let min_diff = 0.2f32;

        // move right
        if diff_x > min_diff && self.pos.0 < max_x - 1 {
            self.pos.0 += 1;
        }
        // move left
        else if diff_x < -min_diff && self.pos.0 > min_x {
            self.pos.0 -= 1;
        }

        // move up
        if diff_y > min_diff && self.pos.1 < max_y - 1 {
            self.pos.1 += 1;
        }
        // move down
        else if diff_y < -min_diff && self.pos.1 > min_y {
            self.pos.1 -= 1;
        }
    }

    fn handle_logic_acc_angle(&mut self, angle_x_deg: f32, angle_y_deg: f32) {
        let max_x = 8;
        let min_x = 0;
        let max_y = 8;
        let min_y = 0;
        let min_angle_deg = 5f32;
        let angle_div = 4f32;

        let angle_x_deg = angle_x_deg / angle_div;
        let angle_y_deg = angle_y_deg / angle_div;

        // move right
        if angle_x_deg > min_angle_deg && self.pos.0 < max_x - 1 {
            self.pos.0 += 1;
        }
        // move left
        else if angle_x_deg < -min_angle_deg && self.pos.0 > min_x {
            self.pos.0 -= 1;
        }

        // move up
        if angle_y_deg > min_angle_deg && self.pos.1 < max_y - 1 {
            self.pos.1 += 1;
        }
        // move down
        else if angle_y_deg < -min_angle_deg && self.pos.1 > min_y {
            self.pos.1 -= 1;
        }
    }

    pub async fn run(&mut self) {
        log::info!("Logic started");

        self.clear_led_matrix().await;
        let mut old_time = SystemTime::now();

        loop {
            let current_time = SystemTime::now();

            if self.old_pos != self.pos {
                self.update_led_matrix().await;
            }

            let acc_data = self.acc_observer.recv().await.unwrap();

            // store old position
            self.old_pos = self.pos;

            if current_time.duration_since(old_time).unwrap().as_millis() > 50 {
                old_time = current_time;

                //self.handle_logic_acc_vec( acc_data.acc_vec.1, -acc_data.acc_vec.0 );

                self.handle_logic_acc_angle( acc_data.acc_angle.0, -acc_data.acc_angle.1 );
            }

            log::info!("Logic:  ({}, {})  v = ( {:1.1} , {:1.1} , {:1.1} )   ang = ( {:1.1} , {:1.1} , {:1.1} )", self.pos.0, self.pos.1, acc_data.acc_vec.0, acc_data.acc_vec.1, acc_data.acc_vec.2, acc_data.acc_angle.0, acc_data.acc_angle.1, acc_data.acc_angle.2);
        }

    }

}
