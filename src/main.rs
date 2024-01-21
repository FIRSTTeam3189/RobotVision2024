use image::{ImageBuffer, Rgba};
use nokhwa::pixel_format::RgbAFormat;

mod camera;
use crate::camera::Camera;

#[cfg(feature = "gui")]
mod gui;

fn main() {
    let (tx, rx) = crossbeam_channel::bounded::<ImageBuffer<Rgba<u8>, Vec<u8>>>(1);

    let mut proc_camera = Camera::new(1, move |frame| {
        let _ = tx.send(frame.decode_image::<RgbAFormat>().unwrap()); 
    }).unwrap();

    proc_camera.start_stream();

    #[cfg(feature = "gui")]
    let _ = eframe::run_native(
        "Vision-App", 
        eframe::NativeOptions::default(),
        Box::new(|_c| Box::new(gui::VisionApp::new(rx)))
    );

    #[cfg(not(feature = "gui"))]
    loop { 
        if false {
            break;
        }
    }

    proc_camera.stop_stream();
}