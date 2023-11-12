use egui::{ColorImage, ImageSource, TextureHandle, TextureId, TextureOptions};
use rand::Rng;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct WorldViewApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for WorldViewApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl WorldViewApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

fn create_texture_from_bitmap(
    ctx: &egui::Context,
    width: usize,
    height: usize,
    data: Vec<u8>,
) -> TextureHandle {
    let image = ColorImage::from_gray([width, height], data.as_ref());
    let texture_handle: TextureHandle =
        ctx.load_texture("random_bitmap", image, TextureOptions::default());
    texture_handle
}

impl eframe::App for WorldViewApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        let bitmap = generate_random_bitmap(128, 128); // Adjust the size as needed
        let texture_id: TextureHandle = create_texture_from_bitmap(&ctx, 128, 128, bitmap);
        let sized_texture = egui::load::SizedTexture {
            id: texture_id.id(),
            size: egui::Vec2::new(200.0, 200.0),
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            // ui.heading("eframe template");
            //
            // ui.horizontal(|ui| {
            //     ui.label("Write something: ");
            //     ui.text_edit_singleline(&mut self.label);
            // });
            //
            // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();
            //
            // ui.add(egui::github_link_file!(
            //     "https://github.com/emilk/eframe_template/blob/master/",
            //     "Source code."
            // ));
            // ui.separator();

            // ui.add( |ui| ui.image(texture_id) );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.image(ImageSource::Texture(sized_texture));
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

// Make sure to include this at the top

fn generate_random_bitmap(width: usize, height: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut data = Vec::with_capacity(width * height);
    for _ in 0..width * height {
        // Randomly choose between black (0) and white (255)
        let value = if rng.gen_bool(0.5) { 255 } else { 0 };
        data.push(value);
    }
    data
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
