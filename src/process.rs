use apriltag::{Detector, DetectorBuilder, Image, TagParams};
use apriltag_image::prelude::*;
use crossbeam_channel::*;
use image::*;
use nalgebra::*;
use bondrewd::Bitfields;
use crate::{config::CameraCalibration, DetectionConfig};

#[derive(Debug, Clone, Bitfields)]
#[bondrewd(default_endianness = "le")]
pub struct VisionData {
    pub detected: bool,
    pub tag_id: u64,
    pub timestamp: f64,
    pub rotation: [f64; 3],
    pub translation: [f64; 3]
}

impl VisionData {
    pub fn new(detected:bool, tag_id: u64, timestamp: f64, rotation:[f64; 3], translation:[f64; 3]) -> Self {
        VisionData {detected, tag_id, timestamp, rotation, translation }
    }
}

pub struct Process {
    image_rx: Receiver<DynamicImage>,
    data_tx: Sender<VisionData>,
    detector: Detector,
    cal: TagParams,
}

impl Process {
    pub fn new(image_rx: Receiver<DynamicImage>, data_tx: Sender<VisionData>, cal: CameraCalibration, config: DetectionConfig) -> Self {
        let detector = DetectorBuilder::new();
        let detector = detector.add_family_bits(&config.families, 1);
        
        let mut detector = detector.build().unwrap();
        detector.set_thread_number(5);
        
        let cal = (&cal).into();

        Process { image_rx, data_tx, detector, cal }
    }

    pub fn update(&mut self) {
        if let Ok(image) = self.image_rx.recv() {
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

                        let _ = self.data_tx.send(VisionData::new(
                            true,
                            tag.id() as u64,
                            0.0,
                            [rotation.0, rotation.1, rotation.2],
                            [transform.x, transform.y, transform.z]
                        ));
                    }
                } else {
                    let _ = self.data_tx.send(VisionData::new(
                        false,
                        0,
                        0.0,
                        [0.0, 0.0, 0.0],
                        [0.0, 0.0, 0.0]
                    ));
                }
            } else {
                let _ = self.data_tx.send(VisionData::new(
                    false,
                    0,
                    0.0,
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0]
                ));
            }
            
        }
    }
}