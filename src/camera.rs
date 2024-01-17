use std::io::Error;

use nokhwa::{CallbackCamera, Buffer, utils::{CameraIndex, RequestedFormat, RequestedFormatType}, pixel_format::RgbAFormat};


pub struct Camera {
    pub callback_cam: CallbackCamera,
    pub index: u32
}

impl Camera {
    pub fn new(callback: impl FnMut(Buffer) + Send + 'static) -> Result<Self, Error> {
        // Setting the Camera Input format
        let format = RequestedFormatType::AbsoluteHighestFrameRate;
        // let format = RequestedFormatType::Exact(
        //     CameraFormat::new(Resolution::new(1920, 1080),
        //     nokhwa::utils::FrameFormat::RAWRGB,
        //     30
        // ));
        let format = RequestedFormat::new::<RgbAFormat>(format);

        // Camera Indexing 
        // TODO: Poll all devices and get Camera indexes
        let index = CameraIndex::Index(0);
        //Creates the camera with given settings
        let x = CallbackCamera::new(index, format, callback).unwrap();
        Ok(Camera {callback_cam: x, index: 0})
    }

    pub fn start_stream(&mut self) {
        let _ = self.callback_cam.open_stream();
    }
}