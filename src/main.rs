use config::*;
use image::DynamicImage;
use process::Process;
use crate::camera::Camera;
use tokio::runtime::Handle;
use std::env;

mod camera;
mod process;
mod config;
mod server;

use server::server::*;

#[cfg(feature = "gui")]
mod gui;

pub const CAL_FILE_NAME: &str = "configs\\cam-cal.json";
pub const CONFIG_FILE_NAME: &str = "configs\\config.json";
pub const SERVER_FILE_NAME: &str = "configs\\server.json";

#[tokio::main]
async fn main() {
    let runtime = Handle::current();
    let env_path = env::current_dir().unwrap();
    
    // Calibration & Config Files
    let calibration = CameraCalibration::load_from_file(env_path.join(CAL_FILE_NAME)).unwrap();
    let config = Config::load_from_file(env_path.join(CONFIG_FILE_NAME)).unwrap();
    let server_config = server::config::ServerConfig::load_from_file(env_path.join(SERVER_FILE_NAME)).unwrap();

    // Creating Channels
    let (tx, rx) = crossbeam_channel::bounded::<DynamicImage>(1);
    let mut server = Server::start(server_config).await.unwrap();
    
    // --------------------- Process Camera ---------------------------
    let mut proc_camera;

    loop {
        if let Ok(cam) = Camera::new(config.camera_index) {
            proc_camera = cam;
            break;
        }
    }
    // -----------------------------------------------------------------

    proc_camera.start_stream();

    // ------------------- Process Thread ------------------------------
    let mut proc_thread = Process::new(rx.clone(), calibration, config.detection_config);

    runtime.spawn(async move{
        proc_camera.callback_thread(tx); 
    });

    // Process Thread
    runtime.spawn(async move {
        loop {
            server.publish();
            proc_thread.update();
        }
    });
    // -----------------------------------------------------------------

    // GUI
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

    // proc_camera.stop_stream();
}