#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

//const COUNT_IMAGES_TO_GRAB: u32 = 10;

use eframe::{egui, epaint::FontFamily};
use egui::{FontData, FontDefinitions};
use pylon_cxx::{InstantCamera, Pylon};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
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
    camera: InstantCamera<'cam>,
    #[allow(dead_code)]
    pylon: Pylon,
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

        Self { camera, pylon }
    }
}

impl<'cam> eframe::App for GrabApp<'cam> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            //pylon
            let dev_info = self
                .camera
                .device_info()
                .model_name()
                .unwrap_or("default".to_string());

            ui.heading("My egui Application");

            ui.label(dev_info);
        });
    }
}
