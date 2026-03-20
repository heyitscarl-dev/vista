fn get_options(dimensions: [f32; 2]) -> eframe::NativeOptions { eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size(dimensions),
    ..Default::default()
}}

fn get_window(name: impl Into<&'static str>, dimensions: [f32; 2]) {
    let options = get_options(dimensions);
    eframe::run_native(name.into(), options, Box::new(|_| {
        Ok(Box::<App>::default())
    })).unwrap();
}

#[derive(Default)]
struct App;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Vista");
        });
    }
}

fn main() {
    env_logger::init();
    get_window("Vista", [480.0, 320.0]);
}
