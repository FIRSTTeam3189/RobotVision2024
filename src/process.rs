use apriltag::{Detector, DetectorBuilder, Image, TagParams};
use apriltag_image::prelude::*;
use crossbeam_channel::*;
use image::*;
use nalgebra::*;

use crate::{config::CameraCalibration, DetectionConfig};

pub struct Process {
    image_rec: Receiver<DynamicImage>,
    detector: Detector,
    cal: TagParams,
}

impl Process {
    pub fn new(recv: Receiver<DynamicImage>, cal: CameraCalibration, config: DetectionConfig) -> Process {
        let detector = DetectorBuilder::new();
        let detector = detector.add_family_bits(&config.families, 1);
        
        let mut detector = detector.build().unwrap();
        detector.set_thread_number(5);
        
        let cal = (&cal).into();

        Process { 
            image_rec: recv,
            detector: detector,
            cal
        }
    }

    pub fn update(&mut self) {
        if let Ok(image) = self.image_rec.recv() {
            let image_buf = Image::from_image_buffer(&image.to_luma8());
            let detections = self.detector.detect(&image_buf);

            if !detections.is_empty() {
                let tag = &detections[0];
                if tag.decision_margin() > 55.0 {
                    if let Some(pose) = tag.estimate_tag_pose(&self.cal) {
                        let mut rotation = Rotation3::from_matrix(&MatrixView3::from_slice(pose.rotation().data()).transpose());
                        rotation.renormalize();
                        let rotation = rotation.euler_angles();
                        println!("Rotation: X: {0}, Y: {1}, Z: {2}", rotation.0.to_degrees(), rotation.1.to_degrees(), rotation.2.to_degrees());

                        let transform: Translation3<f64> = MatrixView3x1::from_slice(pose.translation().data()).into_owned().into();
                        println!("Translation: X:{0} Y:{1} Z: {2}", transform.x, transform.y, transform.z);
                    }
                }
            }
            
        }
    }
}