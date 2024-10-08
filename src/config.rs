use apriltag::*;
use imageproc::geometric_transformations::Projection;
use nalgebra::Matrix3x1;
use serde::*;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CalibrationError {
    #[error("Failed to convert into projection matrix: {0}")]
    ConversionError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Failed to load file: {0}")]
    LoadError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraCalibration {
    /// The camera calibration matrix/Intrinsic camera matrix
    ///
    /// Use the `projection()` to grab the equivalent projection matrix
    mtx: Vec<Vec<f64>>,
    /// The `dist` parameter from the camera calibration script.
    ///
    /// Lens distortion coefficients. Basically whether there are pincushon (think concave) vs barrel (think convex) distortion effects
    dist: Vec<Vec<f64>>,
    /// Per image, the `rvec` or rotation vectors the checkerboard pattern is present.
    ///
    /// Rotation specified as a 3×1 vector. The direction of the vector specifies the axis of rotation and the magnitude of the vector specifies the angle of rotation.
    rvecs: Vec<Vec<Vec<f64>>>,
    /// Per image, the `tvec` or translation vectors the checkerboard pattern is present
    ///
    /// 3×1 Translation vector.
    tvecs: Vec<Vec<Vec<f64>>>,
    /// Focal width in pixels for the camera.
    /// Directly used in the AprilTag detection
    fx: f64,
    /// Focal height in pixels for the camera.
    /// Directly used in the AprilTag detection.
    fy: f64,
    /// Principle focal point of the camera in pixels
    cx: f64,
    /// Principle focal point of the camera in pixels
    cy: f64,
    /// The size of the april tags, in meters
    tagsize: f64,
}

impl Default for CameraCalibration {
    fn default() -> Self {
        Self {
            mtx: vec![vec![]],
            dist: vec![vec![]],
            rvecs: vec![vec![vec![]]],
            tvecs: vec![vec![vec![]]],
            fx: 0.0,
            fy: 0.0,
            cx: 0.0,
            cy: 0.0,
            tagsize: 0.0,
        }
    }
}

impl CameraCalibration {
    /// Loads the calibration JSON file from the given path
    pub fn load_from_file<T: AsRef<Path>>(path: T) -> Result<Self, CalibrationError> {
        let json_text = std::fs::read_to_string(path)?;
        match serde_json::from_str(&json_text) {
            Ok(v) => Ok(v),
            Err(e) => Err(CalibrationError::LoadError(format!("{e}"))),
        }
    }

    /// Principle focal point of the camera in pixels
    pub fn fx(&self) -> f64 {
        self.fx
    }

    /// Principle focal point of the camera in pixels
    pub fn fy(&self) -> f64 {
        self.fy
    }

    /// Focal width in pixels for the camera
    pub fn cx(&self) -> f64 {
        self.cx
    }

    /// Focal height in pixels for the camera
    pub fn cy(&self) -> f64 {
        self.cy
    }

    /// Creates a tag params struct from given calibration
    pub fn tag_params(&self) -> TagParams {
        TagParams {
            cx: self.cx,
            cy: self.cy,
            fx: self.fx,
            fy: self.fy,
            tagsize: self.tagsize,
        }
    }

    /// Lens distortion coefficients. Basically whether there are pincushon (think concave) vs barrel (think convex) distortion effects
    pub fn dist(&self) -> Vec<f64> {
        let dist_flattened: Vec<f64> = self.dist.iter().fold(vec![], |mut acc, v| {
            acc.extend(v.iter());
            acc
        });
        dist_flattened
    }

    /// Returns the vector of rvecs as a Matrix3x1
    pub fn rvecs(&self) -> Result<Vec<Matrix3x1<f64>>, CalibrationError> {
        let mut rvecs = vec![];
        for rvec in self.rvecs.iter() {
            // Fold elements into single vector
            let folded: Vec<f64> = rvec.iter().fold(vec![], |mut acc, v| {
                acc.extend(v);
                acc
            });
            if folded.len() != 3 {
                return Err(CalibrationError::ConversionError(format!(
                    "Incorrect number of elements for rvecs, got {} expected 3",
                    folded.len()
                )));
            }
            let mat = Matrix3x1::from_row_slice(folded.as_slice());
            rvecs.push(mat);
        }

        Ok(rvecs)
    }

    /// Returns the vector of tvecs as a Matrix3x1
    pub fn tvecs(&self) -> Result<Vec<Matrix3x1<f64>>, CalibrationError> {
        let mut tvecs = vec![];
        for tvec in self.tvecs.iter() {
            // Fold elements into single vector
            let folded: Vec<f64> = tvec.iter().fold(vec![], |mut acc, v| {
                acc.extend(v);
                acc
            });
            if folded.len() != 3 {
                return Err(CalibrationError::ConversionError(format!(
                    "Incorrect number of elements for tvecs, got {} expected 3",
                    folded.len()
                )));
            }
            let mat = Matrix3x1::from_row_slice(folded.as_slice());
            tvecs.push(mat);
        }

        Ok(tvecs)
    }

    /// Gets the equivalent projection matrix from `imageproc::geometric_transformations::Projection`
    pub fn projection_mtx(&self) -> Result<Projection, CalibrationError> {
        let flattened: Vec<f32> = self.mtx.as_slice().iter().fold(vec![], |mut acc, v| {
            acc.extend(v.iter().map(|b| *b as f32));
            acc
        });
        let projection_arr: [f32; 9] = match flattened.try_into() {
            Ok(arr) => arr,
            Err(err) => {
                return Err(CalibrationError::ConversionError(format!(
                    "Elements invalid: {err:?}"
                )));
            }
        };

        Projection::from_matrix(projection_arr).ok_or_else(|| {
            CalibrationError::ConversionError(
                "Invalid projection matrix: not invertible".to_string(),
            )
        })
    }
}

impl From<&CameraCalibration> for TagParams {
    fn from(value: &CameraCalibration) -> Self {
        value.tag_params()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DetectionConfig {
    pub families: AprilTagFamily,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub camera_index: u32,
    pub detection_config: DetectionConfig,
    pub interface: InterfaceConfig
}

impl Config {
    pub fn load_from_file<T: AsRef<Path>>(path: T) -> Result<Self, CalibrationError> {
        let json_text = std::fs::read_to_string(path).unwrap();
        match serde_json::from_str(&json_text) {
            Ok(v) => Ok(v),
            Err(_e) => Err(CalibrationError::LoadError(_e.to_string())),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct InterfaceConfig {
    pub nt_ip: [u8; 4],
    pub nt_port: u16,
    pub server_port: u16,
    pub serial_port: String
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum AprilTagFamily {
    #[default]
    Tag16H5,
    Tag25H9,
    Tag36H11,
    TagCircle21H7,
    TagCircle49h12,
    TagStandard41h12,
    TagStandard52h13,
    TagCustom48h12,
}

impl From<&AprilTagFamily> for Family {
    fn from(value: &AprilTagFamily) -> Self {
        match value {
            AprilTagFamily::Tag16H5 => "tag16h5".parse().unwrap(),
            AprilTagFamily::Tag25H9 => "tag25h9".parse().unwrap(),
            AprilTagFamily::Tag36H11 => "tag36h11".parse().unwrap(),
            AprilTagFamily::TagCircle21H7 => "tagCircle21h7".parse().unwrap(),
            AprilTagFamily::TagCircle49h12 => "tagCircle49h12".parse().unwrap(),
            AprilTagFamily::TagStandard41h12 => "tagStandard41h12".parse().unwrap(),
            AprilTagFamily::TagStandard52h13 => "tagStandard52h13".parse().unwrap(),
            AprilTagFamily::TagCustom48h12 => "tagCustom48h12".parse().unwrap(),
        }
    }
}
