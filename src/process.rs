use crossbeam_channel::*;
use image::*;

use crate::config::CameraCalibration;

pub struct Process {
    pub image_rec: Receiver<ImageBuffer<Rgba<u8>, Vec<u8>>>
}

impl Process {
    pub fn new(recv: Receiver<ImageBuffer<Rgba<u8>, Vec<u8>>>, cal: CameraCalibration) -> Process {
        Process { image_rec: recv }
    }

    pub fn update(&self) {
        let _image = self.image_rec.recv().unwrap();


    }
}