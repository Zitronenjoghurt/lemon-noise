mod export;

pub use export::{ExportRequest, ExportWindow};

pub trait Window {
    fn title(&self) -> &str;
    fn is_open(&self) -> bool;
    fn set_open(&mut self, open: bool);
    fn ui(&mut self, ui: &mut egui::Ui);

    fn toggle(&mut self) {
        let open = self.is_open();
        self.set_open(!open);
    }

    fn show(&mut self, ctx: &egui::Context) {
        let mut open = self.is_open();
        egui::Window::new(self.title().to_owned())
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| self.ui(ui));
        self.set_open(open);
    }
}
