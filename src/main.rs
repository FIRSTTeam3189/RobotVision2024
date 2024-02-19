use crate::camera::Camera;
use crate::interface::*;
use crate::process::VisionData;
use config::*;
use image::DynamicImage;
use nokhwa::query;
use process::Process;
use std::env;
use tokio::runtime::Handle;

mod camera;
mod config;
mod process;
mod interface;

#[cfg(feature = "gui")]
mod gui;

pub const CAL_FILE_NAME: &str = "configs/cam-cal.json";
pub const CONFIG_FILE_NAME: &str = "configs/config.json";

#[tokio::main]
async fn main() {
    let runtime = Handle::current();
    let env_path = env::current_dir().unwrap();

    // Calibration & Config Files
    let calibration = CameraCalibration::load_from_file(env_path.join(CAL_FILE_NAME)).unwrap();
    let config = Config::load_from_file(env_path.join(CONFIG_FILE_NAME)).unwrap();

    println!("Loaded Configs!");

    // Creating Channels
    let (image_tx, image_rx) = crossbeam_channel::bounded::<DynamicImage>(1);
    let (data_tx, data_rx) = crossbeam_channel::bounded::<VisionData>(1);
    println!("Created Channels!");

    // ------------------- Server Thread -------------------------------

    runtime.spawn(async move {
        #[cfg(feature = "serial")]
        let mut data_interface = open_serial_port(&config.interface).await.unwrap();

        #[cfg(feature = "server")]
        let mut data_interface = start_tcp_server(&config.interface).await.unwrap();

        println!("Connected to interface!");

        // #[cfg(feature = "nt")]
        // let mut net = NT::new(network_config).await;
        
        loop {
            match data_rx.recv() {
                Ok(data) => {
                    let _ = data_interface.write_vision_data(data).await;
                },
                Err(_) => {

                },
            }
        }
    });

    println!("Comms Thread Started!");

    // ----------------------------------------------------------------

    // --------------------- Process Camera ---------------------------

    println!("Finding Camera...");
    let mut proc_camera;
    let mut cam_id = config.camera_index;

    loop {
        match query(nokhwa::utils::ApiBackend::Auto) {
            Ok(cameras) =>  {
                match cameras[0].index().as_index() {
                    Ok(index) => {
                        cam_id = index;
                    },
                    Err(err) => {
                        println!("Couldn't find camera [{}]", err);
                    }
                }
            },
            Err(err) => {
                println!("Couldn't obtain backend to find camera [{}]", err);
            }
        }
        
        if let Ok(cam) = Camera::new(cam_id) {
            proc_camera = cam;
            break;
        }
    }

    // -----------------------------------------------------------------

    proc_camera.start_stream();

    // Camera Callback Thread
    runtime.spawn(async move {
        proc_camera.callback_thread(image_tx);
    });
    println!("Found Camera! & Stated Callback Thread!");

    // ------------------- Process Thread ------------------------------
    println!("Starting Process Thread!");

    let mut proc_thread = Process::new(
        image_rx.clone(),
        data_tx,
        calibration,
        config.detection_config,
    );

    // Process Thread
    runtime.spawn(async move {
        loop {
            proc_thread.update();
        }
    });

    println!("Started Process Thread!");
    // -----------------------------------------------------------------

    // GUI
    #[cfg(feature = "gui")]
    let _ = eframe::run_native(
        "Vision-App",
        eframe::NativeOptions::default(),
        Box::new(|_c| Box::new(gui::VisionApp::new(image_rx))),
    );

    #[cfg(not(feature = "gui"))]
    loop {
        if false {
            break;
        }
    }

    // proc_camera.stop_stream();
}
