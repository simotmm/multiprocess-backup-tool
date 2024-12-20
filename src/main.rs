mod backup_tool;
mod windowsManagerNEW;
use crate::windowsManagerNEW::chiudi_finestra;
use std::vec::Vec;
use std::{env, thread};
use windowsManagerNEW::{richiesta_conferma};

use backup_tool::{generate_backup_name, get_extensions, get_usb_path, copy_dir, get_src_path, log_cpu_usage};

//
use auto_launch::{AutoLaunchBuilder};
use eframe::egui::Shape::Vec as OtherVec;

mod backup_command;
use backup_command::{start_backup};

use std::path::Path;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc};
use crate::backup_command::second_command;
use crate::windowsManagerNEW::processo_finestra;

fn main(){
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        let arg = &args[1];
        if arg == "w" {
            richiesta_conferma();
            println!("ora mi chiudo (MAIN)");
            std::process::exit(0);
        }
    }

    let exe = env::current_exe();
    let exe_path = exe.unwrap().to_string_lossy().to_string();

    let current_dir = env::current_dir();
    let current_dir_path = current_dir.unwrap().to_string_lossy().to_string();

    let exe2 = env::current_exe();
    let binding = exe2.unwrap();
    let project_root = binding.parent().unwrap().parent().unwrap().parent();

    let conf_name = "conf.txt";
    let file_di_configurazione = project_root.unwrap().join(conf_name).to_string_lossy().to_string();

    println!("current_dir: '{}'.", current_dir_path);
    println!("exe_path: '{}'.", exe_path);
    println!("file_di_configurazione: '{}'.", file_di_configurazione);

    #[cfg(not(target_os = "macos"))] 
    {
        let auto = AutoLaunchBuilder::new()
            .set_app_name("Group23")
            .set_app_path(&exe_path)  //Imposta il percorso dell'applicazione che deve essere avviata automaticamente
            .set_use_launch_agent(false) //commentato: funzione solo per macOS
            .build()
            .unwrap();


        auto.enable().unwrap();
        println!("Autostart enabled: {}", auto.is_enabled().unwrap());
    }

    #[cfg(target_os = "macos")] 
    {
        let _ = AutoLaunchBuilder::new()
            .set_app_name("Group23")
            .set_app_path(&exe_path)
            .set_use_launch_agent(false) 
            .build()
            .unwrap().enable();

        Command::new("osascript") //per non mostrare il terminale (macOS)
            .arg("-e")
            .arg("tell application \"Terminal\" to set visible of front window to false")
            .output()
            .expect("Failed to hide terminal");
    }

    thread::spawn(|| {
        log_cpu_usage();
    });

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);
    let receiver = start_backup(stop_flag_clone);

    get_src_path(&file_di_configurazione); //per debug, da rimuovere

    let mut origine = Option::None;
    let mut usb_path = Option::None;
    let mut estensioni = vec![];
    let mut destinazione= String::new();

    loop {
        println!("Traccia un rettangolo con il mouse per iniziare il backup...");
        match receiver.recv() { // .recv() blocca finché non riceve un valore
            Ok(success) => {
                if success {
                    origine = get_src_path(&file_di_configurazione);
                    usb_path = get_usb_path();

                    if origine.is_some() && usb_path.is_some() {
                        estensioni = get_extensions(&file_di_configurazione).unwrap();
                        destinazione = generate_backup_name(&origine.clone().unwrap().to_string(), &usb_path.clone().unwrap().to_string());
                    }
                    else {
                        println!("Il backup non è andato a buon fine, riavvio della procedura.");
                        continue; //ricomincia il loop
                    }
                    stop_flag.store(true, Ordering::Relaxed); // Imposta il flag su `true`
                    println!("Flag di stop impostato!");

                    let mut finestra = processo_finestra().unwrap();
                    let second_receiver = second_command();


                    match finestra.try_wait() {
                        Ok(Some(_)) => { //il processo della finestra non è più attivo (è stato chiuso dal tasto)
                            continue;    //ricomincia il loop
                        }
                        Ok(None) => {// Il processo finestra è ancora attivo
                            match second_receiver.recv() {
                                Ok(success) => {
                                    if success {
                                        let _ = chiudi_finestra(finestra);
                                        println!("Rettangolo rilevato! Inizio backup...");
                                        let result = copy_dir(&origine.unwrap().to_string(), &destinazione, estensioni);
                                        println!("{} byte copiati.", result.unwrap());
                                        break; //esce dal loop dopo aver terminato il backup, il programma termina.
                                    }
                                }
                                Err(e) => {
                                    println!("Errore nella comunicazione con il thread (2° comando): {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Errore nel controllare lo stato del processo finestra: {}", e);
                            continue; // In caso di errore, continua il ciclo principale
                        }
                    }
                } else {
                    println!("Operazione fallita.");
                }
            }
            Err(e) => {
                println!("Errore nella comunicazione con il thread: {}", e);
            }
        }
    }

    //let _risultato = copy_dir(&origine, &destinazione, estensioni);

}
