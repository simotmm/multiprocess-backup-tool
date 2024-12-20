/* CARGO.TOML *
[package]
name = "windowsManager"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe="0.29.1"
*/

extern crate eframe;
use eframe::egui;
use std::sync::mpsc::Sender; //nuova

fn main() {
richiesta_conferma();
}

pub fn richiesta_conferma(){
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_resizable(false).with_max_inner_size([300.0, 200.0]),
        centered: true,
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Conferma backup",
        options,
        Box::new(|_cc| Ok(Box::new(ConfirmWindow))),
    );
}

struct ConfirmWindow;

impl eframe::App for ConfirmWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label("\nGesture riconosciuta!\nAvviare un Backup di Emergenza?");
                    ui.vertical_centered_justified(|ui|{
                        ui.label("          ");
                        if ui.button("\n             Avvia backup             \n").clicked() {
                            print!("Avviato!!");
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        };
                        ui.label("          ");
                        if ui.button("\n             Non avviare             \n").clicked() {
                            print!("Annullato!");
                            std::process::exit(0)}
                    }
                    )}
                )
        });
    }
}
