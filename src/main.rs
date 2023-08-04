#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

//const COUNT_IMAGES_TO_GRAB: u32 = 1000;

use eframe::{
    egui,
    epaint::{ColorImage, FontFamily},
};
use egui::{FontData, FontDefinitions};
use itertools::{izip, Itertools};
use log::debug;
use pylon_cxx::{InstantCamera, Pylon};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1300.0, 1000.0)),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        //Box::new(|cc| Box::<MyApp>::default(cc)),
        Box::new(|cc| Box::new(GrabApp::new(cc))),
    )
}

fn setup_jp_fonts(ctx: &egui::Context) {
    //font
    let mut font = FontDefinitions::default();
    // install
    font.font_data.insert(
        "meiryo".to_owned(),
        FontData::from_static(include_bytes!("/Windows/Fonts/meiryo.TTC")),
    );
    // put the font first
    font.families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "meiryo".to_owned());
    // put the font as last fallback
    font.families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push(r"meiryo".to_owned());

    ctx.set_fonts(font);
}

struct GrabApp<'cam> {
    // メモリを開放して欲しい順？にフィールドを記述
    //texture: Option<egui::TextureHandle>,
    image_buffer: Vec<u8>,
    width: u32,
    height: u32,
    pixel_format: String,
    model_name: String,
    camera: InstantCamera<'cam>,
    #[allow(dead_code)]
    pylon: Pylon,
    init: bool,
}

impl<'a, 'cam> GrabApp<'cam> {
    fn new(cc: &eframe::CreationContext<'a>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        setup_jp_fonts(&cc.egui_ctx);

        // dark theme
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let pylon = pylon_cxx::Pylon::new();
        let lefp: &'cam _ = unsafe { &*(&pylon as *const _) };
        let camera = pylon_cxx::TlFactory::instance(lefp)
            .create_first_device()
            .unwrap();

        Self {
            //texture: None,
            image_buffer: vec![0],
            width: 0,
            height: 0,
            pixel_format: "unknown".to_string(),
            model_name: "none".to_string(),
            camera,
            pylon,
            init: false,
        }
    }
}

impl<'cam> eframe::App for GrabApp<'cam> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        debug!("update");

        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            //pylon init
            if !self.init {
                self.init = true;

                self.model_name = self
                    .camera
                    .device_info()
                    .model_name()
                    .unwrap_or("default".to_string());

                let _res = self.camera.open();

                let _res = self
                    .camera
                    .start_grabbing(&pylon_cxx::GrabOptions::default()); //.count(COUNT_IMAGES_TO_GRAB));

                match self.camera.node_map().enum_node("PixcelFormat") {
                    Ok(node) => {
                        self.pixel_format = node.value().unwrap_or("not readable".to_string())
                    }
                    Err(e) => eprintln!("Ignoring error getting PixelFormat node: {}", e),
                }
            }

            let mut grabbed = false;

            // grab
            let mut grab_result = pylon_cxx::GrabResult::new().unwrap();
            if self.camera.is_grabbing() {
                let _res = self.camera.retrieve_result(
                    5000,
                    &mut grab_result,
                    pylon_cxx::TimeoutHandling::ThrowException,
                );

                if grab_result.grab_succeeded().unwrap_or(false) {
                    self.width = grab_result.width().unwrap_or(0);
                    self.height = grab_result.height().unwrap_or(0);
                }

                if grab_result.buffer_size().unwrap_or(0) > 0 {
                    let image_buffer = grab_result.buffer().unwrap();
                    self.image_buffer.clear();
                    self.image_buffer.extend_from_slice(image_buffer);
                }

                grabbed = true;
            }

            ui.heading("My egui + basler pylon Application");

            ui.label(format!("model name: {}", self.model_name));
            ui.label(format!("pixel format: {}", self.pixel_format));
            ui.label(format!("size: {} x {}", self.width, self.height));
            ui.label(format!("fist pixel: {}", self.image_buffer[0]));

            debug!(
                "size {}x{}, len {}",
                self.width,
                self.height,
                self.image_buffer.len()
            );

            if grabbed {
                // gray to RGBa
                let v255 = vec![255_u8; self.image_buffer.len()];

                let color_buffer = izip!(
                    &self.image_buffer,
                    &self.image_buffer,
                    &self.image_buffer,
                    &v255
                )
                .map(|(&r, &g, &b, &a)| [r, g, b, a])
                .collect_vec()
                .concat();

                let texture = &ui.ctx().load_texture(
                    "camera",
                    ColorImage::from_rgba_unmultiplied(
                        [self.width as _, self.height as _],
                        color_buffer.as_slice(),
                    ),
                    Default::default(),
                );

                ui.image(texture, texture.size_vec2());
            }
        });
    }
}
