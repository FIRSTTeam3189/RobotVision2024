use egui::{ColorImage, TextureHandle}; 
use eframe::egui;
use image::{ImageBuffer, Rgba, DynamicImage};
use crossbeam_channel::*;

pub struct VisionApp {
    image: Option<ColorImage>,
    texture: Option<TextureHandle>,
    image_receiver: Receiver<DynamicImage>,
}

impl VisionApp {
    pub fn new(image_receiver: Receiver<DynamicImage>) -> VisionApp {
        VisionApp {
            image: None,
            texture: None,
            image_receiver,
        }
    }
}

impl eframe::App for VisionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            if let Ok(buffer) = self.image_receiver.recv() {
                let size = [buffer.width() as _, buffer.height() as _];
                let buffer = buffer.as_flat_samples_u8().unwrap();
                let image = ColorImage::from_rgba_unmultiplied(size, &buffer.as_slice());
                self.image = Some(image);
            }

            if let Some(frame) = self.image.take() {
                self.texture = Some(ui.ctx().load_texture("frame", frame, Default::default()))
            }

            if let Some(texture) = self.texture.as_ref() {
                ui.image((texture.id(), ui.available_size()));
            } else {
                ui.spinner();
            }
    
            ctx.request_repaint();
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        std::process::exit(0);
    }

}