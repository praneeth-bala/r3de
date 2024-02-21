#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use std::sync::{ atomic::Ordering, Arc, Mutex };
use r3de::objs::{ GUIState, DisplayBuffers };
use r3de::engine::Engine;
use std::time::Instant;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "R3DE",
        options,
        Box::new(|cc| {
            Box::new(R3DE::new(cc))
        }),
    )
}

struct R3DE {
    // state: Arc<Mutex<GUIState>>, 
    buffers: DisplayBuffers,
    time: Instant,
    frames: f64,
}

impl R3DE {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = Arc::new(Mutex::new(GUIState::new()));
        state.lock().unwrap().ctx = Some(cc.egui_ctx.clone());

        let buf_size = [700 as usize, 700 as usize];
        let buffers = DisplayBuffers::new(buf_size);
        
        let mut engine = Engine::new(state.clone(), &buffers);
        std::thread::spawn(move ||{Engine::lo(&mut engine)});

        Self {
            // state, 
            buffers,
            time: Instant::now(),
            frames: 0.0,
        }
    }
}

impl eframe::App for R3DE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut trip_state_lock = self.buffers.trip_state.lock().unwrap();
        if !self.buffers.stale.load(Ordering::Relaxed) {
            (trip_state_lock[0], trip_state_lock[2]) = (trip_state_lock[2], trip_state_lock[0]);
            self.buffers.stale.store(true, Ordering::Relaxed);
        }
        let paint_buffer_index = trip_state_lock[2];
        drop(trip_state_lock);

        egui::CentralPanel::default().show(ctx, |ui| {

            let buf_lock = self.buffers.bufs[paint_buffer_index as usize].lock().unwrap();
            let img = ui.ctx().load_texture(
                "fg",
                egui::ImageData::Color(egui::ColorImage { size: self.buffers.buf_size, pixels: buf_lock.to_vec()}.into()),
                Default::default()
            );
            drop(buf_lock);
            ui.add(egui::Image::new(&img));

            let elapsed = self.time.elapsed().as_millis();
            self.frames += 1.0;
            ui.label(format!("FPS: {}", (self.frames/elapsed as f64)*(1000 as f64)));
            if elapsed >1000 {
                self.frames = 0.0;
                self.time = Instant::now();
            }

        });
    }
}