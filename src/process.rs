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

    // Rotation Data 0 gives [0 on left, 1 on center, 0 on right] 
    // Rotation Data 1 gives [1 Rotated to the left, -1 Rotated to the right]
    // Rotation Data 2 gives [0 Upright, 1 upside down]
    // Rotation Data 3 gives [1 Tag Rotated to the right, -1 Tag Rotated to the left]
    // Rotation Data 4 gives [0 top, 0 bottom, 1 center]
    // Rotation Data 5 gives [-1 top, 1 bottom, 0 center]
    // Rotation Data 6 gives [1 left, 0 center, -1 right]
    // Rotation Data 7 gives [-1 Bottom, 0 center, 1 top]
    // Rotation Data 8 gives [0 top, 0 bottom, 1 center]

    pub fn update(&mut self) {
        if let Ok(image) = self.image_rec.recv() {
            let image_buf = Image::from_image_buffer(&image.to_luma8());
            let detections = self.detector.detect(&image_buf);
            for tag in detections {
                if tag.decision_margin() > 55.0 {
                    if let Some(pose) = tag.estimate_tag_pose(&self.cal) {
                        println!("Rotation: {0}", pose.rotation().data()[6]);
                        // let _rotation_matrix = pose.rotation().data()[6] * 90.0; /*Z-Rot:*/
                        // let _translation_matrix = [pose.translation().data()[2], pose.translation().data()[0], pose.translation().data()[1]];
                    }
                }+
            }
        }
    }
}