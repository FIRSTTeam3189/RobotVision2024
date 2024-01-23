use config::*;
use image::{ImageBuffer, Rgba};
use nokhwa::pixel_format::RgbAFormat;
use process::Process;
use crate::camera::Camera;
use tokio::runtime::Handle;
use std::env;

mod camera;
mod process;
mod config;

#[cfg(feature = "gui")]
mod gui;

#[tokio::main]
async fn main() {
    let runtime = Handle::current();
    let env_path = env::current_dir().unwrap();
    
    // Calibration & Config Files
    let calibration = CameraCalibration::load_from_file(env_path.join(CAL_FILE_NAME)).unwrap();
    let config = Config::load_from_file(env_path.join(CONFIG_FILE_NAME)).unwrap();

    // Creating Channels
    let (tx, rx) = crossbeam_channel::bounded::<ImageBuffer<Rgba<u8>, Vec<u8>>>(1);

    let mut proc_camera = Camera::new(config.camera_index, move |frame| {
        let _ = tx.send(frame.decode_image::<RgbAFormat>().unwrap());
    }).unwrap();

    let proc_thread = Process::new(rx.clone(), calibration, config.detection_config);

    proc_camera.start_stream();

    // Process Thread
    runtime.spawn(async move {
        loop {
            proc_thread.update();
        }
    });

    #[cfg(feature = "gui")]
    let _ = eframe::run_native(
        "Vision-App", 
        eframe::NativeOptions::default(),
        Box::new(|_c| Box::new(gui::VisionApp::new(rx)))
    );

    #[cfg(not(feature = "gui"))]
    loop {
        if false { break; }
    }

    proc_camera.stop_stream();
}