

extern crate eframe;

use std::process::Child;
use std::env;
use std::process::Command;
use eframe::{egui, App};

pub fn processo_finestra() -> Option<Child> {
    let exe = env::current_exe();
    let exe_path = exe.unwrap().to_string_lossy().to_string();
    let child= Command::new(exe_path).arg("w").spawn();
    match child {
        Ok(mut child) => {
            Some(child)
        }
        Err(e) => {
            println!("Errore nell'avvio del processo: {}", e);
            None
        }
    }
}

pub fn chiudi_finestra(mut child: Child) -> Result<(), String> {
    match child.kill() {
        Ok(_) => {
            if let Err(e) = child.wait() {
                return Err(format!("Errore nell'attendere la terminazione del processo: {}", e));
            }
            Ok(())
        }
        Err(e) => Err(format!("Errore nel terminare il processo: {}", e)),
    }
}

pub fn richiesta_conferma(){
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_resizable(false).with_max_inner_size([300.0, 120.0]),
        centered: true,
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Avviare Backup di Emergenza?",
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
                    ui.label("\nGesture riconosciuta!\nConfermare con la successiva o annullare");
                    ui.vertical_centered_justified(|ui| {

                        ui.label("          ");
                        if ui.button("\n             Non avviare             \n").clicked() {
                            print!("Annullato!");
                            println!("ORA MI CHIUDO (winmanager)");
                            std::process::exit(0);
                        }
                    }
                    )
                }
                )
            });
    }
}
