//! Minimal GUI front-end: a Save and a Restore button, each opening the
//! native file dialog, plus a one-line status.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() -> eframe::Result {
    gui::run()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("deskshot-gui only supports Windows");
}

#[cfg(windows)]
mod gui {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{self, Receiver};
    use std::thread;

    use deskshot::app::restore::RestoreReport;
    use deskshot::composition;
    use eframe::egui;

    pub fn run() -> eframe::Result {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([300.0, 80.0])
                .with_resizable(false),
            ..Default::default()
        };
        eframe::run_native("deskshot", options, Box::new(|_| Ok(Box::<DeskshotApp>::default())))
    }

    /// Capture and restore run on a worker thread (restore can wait many
    /// seconds for apps to start); the result comes back as a status line.
    #[derive(Default)]
    struct DeskshotApp {
        status: String,
        job: Option<Receiver<String>>,
    }

    impl DeskshotApp {
        fn start(&mut self, ctx: &egui::Context, work: impl FnOnce() -> String + Send + 'static) {
            let (sender, receiver) = mpsc::channel();
            let ctx = ctx.clone();
            thread::spawn(move || {
                let _ = sender.send(work());
                ctx.request_repaint();
            });
            self.status = "Working…".into();
            self.job = Some(receiver);
        }

        fn collect_finished_job(&mut self) {
            if let Some(message) = self.job.as_ref().and_then(|job| job.try_recv().ok()) {
                self.status = message;
                self.job = None;
            }
        }
    }

    impl eframe::App for DeskshotApp {
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            self.collect_finished_job();
            let ctx = ui.ctx().clone();
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add_enabled_ui(self.job.is_none(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Save…").clicked() {
                            if let Some(path) = dialog().set_file_name("layout.json").save_file() {
                                self.start(&ctx, move || save(&path));
                            }
                        }
                        if ui.button("Restore…").clicked() {
                            if let Some(path) = dialog().pick_file() {
                                self.start(&ctx, move || restore(&path));
                            }
                        }
                    });
                });
                ui.label(&self.status);
            });
        }
    }

    fn dialog() -> rfd::FileDialog {
        rfd::FileDialog::new().add_filter("deskshot layout", &["json"])
    }

    fn save(path: &PathBuf) -> String {
        match composition::capture(path) {
            Ok(snapshot) => format!("Saved {} windows to {}", snapshot.windows.len(), name(path)),
            Err(error) => format!("Error: {error}"),
        }
    }

    fn restore(path: &PathBuf) -> String {
        match composition::restore(path) {
            Ok(report) => summary(&report),
            Err(error) => format!("Error: {error}"),
        }
    }

    fn summary(report: &RestoreReport) -> String {
        let problems =
            report.launch_failures.len() + report.placement_failures.len() + report.missing.len();
        match problems {
            0 => format!("Restored {} windows", report.restored),
            n => format!("Restored {} windows, {n} could not be restored", report.restored),
        }
    }

    fn name(path: &Path) -> String {
        path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into())
    }
}
