mod camera;
use crate::camera::Camera;

fn main() {
    let mut proc_camera = Camera::new(callback).unwrap();
    proc_camera.start_stream();
    loop{}
}

fn callback(_frame: nokhwa::Buffer) {
    println!("Callback!");
}
