use std::time::{Duration, UNIX_EPOCH};

use crate::{config::CameraCalibration, DetectionConfig};
use apriltag::{Detector, DetectorBuilder, Image, TagParams};
use apriltag_image::prelude::*;
use bondrewd::Bitfields;
use crossbeam_channel::*;
use image::*;
use nalgebra::*;

#[derive(Debug, Clone, Bitfields)]
#[bondrewd(default_endianness = "le", enforce_bytes = 65)]
pub struct VisionData {
    #[bondrewd(bit_length = 8)]
    pub detected: bool,
    pub tag_id: u64,
    pub timestamp: f64,
    pub translation: [f64; 3],
    pub rotation: [f64; 3],
}

impl VisionData {
    pub fn new(
        detected: bool,
        tag_id: u64,
        timestamp: f64,
        translation: [f64; 3],
        rotation: [f64; 3],
    ) -> Self {
        VisionData {
            detected,
            tag_id,
            timestamp,
            translation,
            rotation
        }
    }
}

pub struct Process {
    image_rx: Receiver<DynamicImage>,
    data_tx: Sender<VisionData>,
    detector: Detector,
    cal: TagParams,
}

impl Process {
    pub fn new(
        image_rx: Receiver<DynamicImage>,
        data_tx: Sender<VisionData>,
        cal: CameraCalibration,
        config: DetectionConfig,
    ) -> Self {
        let detector = DetectorBuilder::new();
        let detector = detector.add_family_bits(&config.families, 1);

        let mut detector = detector.build().unwrap();
        detector.set_thread_number(5);

        let cal = (&cal).into();

        Process {
            image_rx,
            data_tx,
            detector,
            cal,
        }
    }

    pub fn update(&mut self) {
        if let Ok(image) = self.image_rx.recv() {
            let image_buf = Image::from_image_buffer(&image.to_luma8());
            let detections = self.detector.detect(&image_buf);
            let timestamp = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_micros(1)).as_secs_f64();

            if !detections.is_empty() {
                let tag = &detections[0];
                if tag.decision_margin() > 55.0 {
                    if let Some(pose) = tag.estimate_tag_pose(&self.cal) {
                        let mut rotation = Rotation3::from_matrix(
                            &MatrixView3::from_slice(pose.rotation().data()).transpose(),
                        );
                        rotation.renormalize();
                        let rotation = rotation.euler_angles();

                        let transform: Translation3<f64> =
                            MatrixView3x1::from_slice(pose.translation().data())
                                .into_owned()
                                .into();

                        let _ = self.data_tx.send(VisionData::new(
                            true,
                            tag.id() as u64,
                            timestamp,
                            [transform.x, transform.y, transform.z],
                            [rotation.0, rotation.1, rotation.2],
                        ));
                    }
                } else {
                    let _ = self.data_tx.send(VisionData::new(
                        false,
                        0,
                        timestamp,
                        [0.0, 0.0, 0.0],
                        [0.0, 0.0, 0.0],
                    ));
                }
            } else {
                let _ = self.data_tx.send(VisionData::new(
                    false,
                    0,
                    timestamp,
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                ));
            }
        }
    }
}
