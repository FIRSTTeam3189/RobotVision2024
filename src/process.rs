use apriltag::{detector, DetectorBuilder, families::ApriltagFamily};
use apriltag_sys::*;
use crossbeam_channel::*;
use image::*;

use crate::{config::CameraCalibration, DetectionConfig, AprilTagFamily};

pub struct Process {
    image_rec: Receiver<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    calibration: CameraCalibration,
    config: DetectionConfig
}

impl Process {
    pub fn new(recv: Receiver<ImageBuffer<Rgba<u8>, Vec<u8>>>, cal: CameraCalibration, config: DetectionConfig) -> Process {
        let detector = DetectorBuilder::new();
        let detector = config.families.iter().fold(detector, |d, f| d.add_family_bits::<dyn ApriltagFamily>(f.into(), 1));
        Process { 
            image_rec: recv,
            calibration: cal,
            config
        }
    }

    pub fn update(&self) {
        let _image = self.image_rec.recv().unwrap();

        
    }
}