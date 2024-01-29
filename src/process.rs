use apriltag::{Detector, DetectorBuilder, Image, TagParams};
use apriltag_image::prelude::*;
use crossbeam_channel::*;
use image::*;

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
            for tag in detections {
                if tag.decision_margin() > 55.0 {
                    if let Some(pose) = tag.estimate_tag_pose(&self.cal) {
                        println!("Translation: x: {0}, y: {1}, z: {2}", pose.translation().data()[0], pose.translation().data()[1], pose.translation().data()[2]);
                        println!("Rotation: x: {0}, y: {1}, z: {2}", pose.rotation().data()[0], pose.rotation().data()[1], pose.rotation().data()[2]);

                        let _translation_matrix = [pose.translation().data()[2], pose.translation().data()[0], pose.translation().data()[1]];
                    }
                    println!("{0}, trust: {1}", tag.id(), tag.decision_margin());
                }
            }
        }
    }
}