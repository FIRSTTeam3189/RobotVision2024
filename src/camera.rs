use nokhwa::{CallbackCamera, Buffer, utils::{RequestedFormat, RequestedFormatType, Resolution, CameraFormat}, pixel_format::RgbFormat};

pub struct Camera {
    pub callback_cam: CallbackCamera,
    pub index: u32
}

impl Camera {
    pub fn new(index: u32, callback: impl FnMut(Buffer) + Send + 'static) -> Result<Self, ()> {
        // Setting the Camera Input format
        // let format = RequestedFormatType::AbsoluteHighestResolution;
        let format = RequestedFormatType::Exact(
            CameraFormat::new(Resolution::new(1920, 1080),
            nokhwa::utils::FrameFormat::MJPEG,
            30
        ));

        let format = RequestedFormat::new::<RgbFormat>(format);

        //Creates the camera with given settings
        match CallbackCamera::new(nokhwa::utils::CameraIndex::Index(index), format, callback) {
            Ok(camera) =>{
                Ok(Camera {callback_cam: camera, index })
            },
            Err(_err) => {
                Err(())
            },
        }
    }

    pub fn start_stream(&mut self) {
        let _ = self.callback_cam.open_stream();
    }

    pub fn stop_stream(&mut self) {
        let _ = self.callback_cam.stop_stream();
    }
}