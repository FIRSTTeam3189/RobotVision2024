use crossbeam_channel::Sender;
use image::DynamicImage;
use nokhwa::{Camera as Cam, utils::{RequestedFormat, RequestedFormatType, Resolution, CameraFormat}, pixel_format::{RgbAFormat, RgbFormat}};

pub struct Camera {
    pub camera: Cam,
    pub index: u32
}

impl Camera {
    pub fn new(index: u32) -> Result<Self, ()> {
        // Setting the Camera Input format
        // let format = RequestedFormatType::AbsoluteHighestResolution;
        let format = RequestedFormatType::Exact(
            CameraFormat::new(Resolution::new(1920, 1080),
            nokhwa::utils::FrameFormat::MJPEG,
            30
        ));

        let format = RequestedFormat::new::<RgbFormat>(format);

        //Creates the camera with given settings
        match Cam::new(nokhwa::utils::CameraIndex::Index(index), format) {
            Ok(mut camera) =>{
                let _ = camera.set_camera_control(nokhwa::utils::KnownCameraControl::Brightness, nokhwa::utils::ControlValueSetter::Integer(100));
                let _ = camera.set_camera_control(nokhwa::utils::KnownCameraControl::Exposure, nokhwa::utils::ControlValueSetter::Integer(0));
                let _ = camera.set_camera_control(nokhwa::utils::KnownCameraControl::Gain, nokhwa::utils::ControlValueSetter::Integer(100));
                Ok(Camera {camera, index })
            },
            Err(_err) => {
                Err(())
            },
        }
    }

    pub fn start_stream(&mut self) {
        let _ = self.camera.open_stream();
    }

    pub fn _stop_stream(&mut self) {
        let _ = self.camera.stop_stream();
    }

    pub fn callback_thread(&mut self, tx: Sender<DynamicImage>) {
        loop {
            if let Ok(frame) = self.camera.frame(){
                let image = frame.decode_image::<RgbAFormat>().unwrap();
                let image = DynamicImage::from(image);
                let _ = tx.send(image);
            }
        }
    }   
}