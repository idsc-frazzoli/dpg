#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::{CentralPanel, Context, SidePanel, Window};

use dpg::WorldViewApp;

// fn main() {
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();
//     let state = State::new(&window);
//     let world = World::new();
//
//     event_loop.run(move |event, _, control_flow| {
//         *control_flow = ControlFlow::Wait;
//
//         match event {
//             Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
//                 *control_flow = ControlFlow::Exit
//             }
//             Event::RedrawRequested(_) => {
//                 state.begin_frame(&window);
//                 world.render(&state.ctx);
//                 state.end_frame(&window);
//             }
//             _ => (),
//         }
//     });
// }
//

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        initial_window_size: Some([400.0, 300.0].into()),
        min_window_size: Some([300.0, 220.0].into()),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(WorldViewApp::new(cc))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
