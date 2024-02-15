use config::*;
use image::DynamicImage;
use process::Process;
use tokio::net;
use crate::camera::Camera;
use crate::network::*;
use crate::process::VisionData;
use tokio::runtime::Handle;
use std::env;

mod camera;
mod process;
mod config;
mod server;
mod network;

#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "nt")]
mod nt;

#[cfg(not(feature = "nt"))]
mod server;
mod interface;

pub const CAL_FILE_NAME: &str = "configs/cam-cal.json";
pub const CONFIG_FILE_NAME: &str = "configs/config.json";
pub const SERVER_FILE_NAME: &str = "configs/network.json";

#[tokio::main]
async fn main() {
    let runtime = Handle::current();
    let env_path = env::current_dir().unwrap();
    
    // Calibration & Config Files
    let calibration = CameraCalibration::load_from_file(env_path.join(CAL_FILE_NAME)).unwrap();
    let config = Config::load_from_file(env_path.join(CONFIG_FILE_NAME)).unwrap();
    let network_config = NetworkConfig::load_from_file(env_path.join(SERVER_FILE_NAME)).unwrap();
    println!("Loaded Configs!");

    // Creating Channels
    let (image_tx, image_rx) = crossbeam_channel::bounded::<DynamicImage>(1);
    let (data_tx, data_rx) = crossbeam_channel::bounded::<VisionData>(1);
    println!("Created Channels!");

    // ------------------- Server Thread -------------------------------
   let mut network = Network::new(network_config, data_rx);
   network.update(&runtime).await;
   
    println!("Network Thread Started!");

    // ----------------------------------------------------------------
    
    // --------------------- Process Camera ---------------------------


    println!("Finding Camera...");
    let mut proc_camera;

    loop {
        if let Ok(cam) = Camera::new(config.camera_index) {
            proc_camera = cam;
            break;
        }
    }

    // -----------------------------------------------------------------

    proc_camera.start_stream();

    // Camera Callback Thread
    runtime.spawn(async move{
        proc_camera.callback_thread(image_tx); 
    });
    println!("Found Camera! & Stated Callback Thread!");
    // ------------------- Process Thread ------------------------------
    println!("Starting Process Thread!");
    let mut proc_thread = Process::new(image_rx.clone(), data_tx, calibration, config.detection_config);

    // Process Thread
    runtime.spawn(async move {
        loop {
            //proc_thread.update();
        }
    });
    println!("Started Process Thread!");
    // -----------------------------------------------------------------

    // GUI
    #[cfg(feature = "gui")] 
    let _ = eframe::run_native(
        "Vision-App", 
        eframe::NativeOptions::default(),
        Box::new(|_c| Box::new(gui::VisionApp::new(image_rx)))
    );

    #[cfg(not(feature = "gui"))]
    loop {
        if false { break; }
    }

    // proc_camera.stop_stream();
}