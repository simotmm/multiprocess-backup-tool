use std::thread;
use device_query::{DeviceQuery, DeviceState}; //device_query: libreria per chiedere dello stato di mouse e tastiera SENZA bisogno di finestra attiva. funziona per Windows, Mac, Linuz
use rdev::display_size;            //rdev: libreria per sentire/inviare eventi a tastiera/mouse su Windows, Mac, Linux
use std::sync::mpsc::{self};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use eframe::egui::debug_text::print;

pub fn start_backup(stop_flag: Arc<AtomicBool>) -> std::sync::mpsc::Receiver<bool> {
    let (sender, receiver) = mpsc::channel();

    let device_state = DeviceState::new();
    let mut sides: Vec<char> = Vec::with_capacity(4);

    //tutto qui dentro
    thread::spawn(move || {     //thread::spawn crea il thread
                                //move: cioè tutte le variabili catturate dalla closure vengono TRASFERITE a questo nuovo thread (invece di prenderlo in prestito)
                                //quindi il thread principale non può più accedere a quelle variabili dopo che questo thread è stato creato
                                //closure: blocco di codice anonimo (?) eseguito all'interno del thread. è qui dentro che puoi usare le "variabili catturate"
                                //in pratica posso usare le variabili passati come argomento della funzione start_backup

        let (width, height) = display_size().unwrap(); //ritorna dimensione in pixel dello schermo principale
        let w = width as f64;
        let h = height as f64;
        
        let mut is_drawing = false;
        let mut start: (i32, i32) = (0, 0);
        let mut end: (i32, i32);

        loop {
            if stop_flag.load(Ordering::Relaxed){
                println!("interruzione del thread");
                break;
            }

            let mouse = device_state.get_mouse(); //per ottenere le coordinate del mouse e lo stato dei bottoni del mouse
            let coordinates = mouse.coords;
            if mouse.button_pressed[1]{     //pulsante premuto
                if !is_drawing{              //se sta disegnando
                    is_drawing = true;
                    start = coordinates;
                }
            } else{             //pulsante non premuto
                if is_drawing{   //ma lo stato dice che sta disegnando
                    is_drawing = false; //aggiorno lo stato al valore corretto
                    end = coordinates;
                    
                    if sides.len() == 0 {   //inserimento primo segmento
                        if vertical_check(start, end, h){
                            //println!("ho passato il check");
                            sides.push('V');
                            println!("sides: {}", sides.len());
                        }else{
                            if horizontal_check(start, end, w){
                                sides.push('H');
                            }
                        }
                    } else{ //ora vanno fatti i casi in cui un segmento è già stato inserito
                        if sides.len() < 4 {
                            if sides[sides.len() - 1] == 'V' && horizontal_check(start, end, w){
                                sides.push('H');
                                //println!("ho passato il check");
                                println!("sides: {}", sides.len());
                            } else{
                                if sides[sides.len() - 1] == 'H' && vertical_check(start, end, h){
                                    sides.push('V');
                                    //println!("ho passato il check");
                                    println!("sides: {}", sides.len());
                                } else{
                                    sides.clear(); //se ho una V e V oppure H e H oppure un segmento non valido, resetto tutto
                                }
                            }
                        }else{
                            if sides.len() == 4 { //rettangolo fatto
                                /* finestra di conferma qui */
                                println!("ho TEORICAMENTE finito");
                                println!("sides: {}", sides.len());
                                sides.clear();
                                sender.send(true).unwrap();

                                    //break; //si esce dal thread //commentato per permettere di ricominciare in caso di errore, il thread termina
                                                                  //con il break nel main insieme al processo principale (continua a rilevare il
                                                                  //mouse durante finestra e backup, flag/messaggio per metterlo in pausa?)
                            }
                        }
                    } 
                }
            }
        }
    });

    receiver
}

pub fn second_command() -> std::sync::mpsc::Receiver<bool> {
    println!("second command start");
    let (sender, receiver) = mpsc::channel();
    let device_state = DeviceState::new();
    thread::spawn(move || {
        let (width, height) = display_size().unwrap(); //ritorna dimensione in pixel dello schermo principale
        let w = width as f64;
        let h = height as f64;

        let mut is_drawing = false;
        let mut start: (i32, i32) = (0, 0);
        let mut end: (i32, i32) = (0,0);
        loop {
            //println!("{}, {}, {}, {}", start.0, start.1, end.0, end.1);
            let mouse = device_state.get_mouse(); //per ottenere le coordinate del mouse e lo stato dei bottoni del mouse
            let coordinates = mouse.coords;
            if mouse.button_pressed[1]{     //pulsante premuto
                if !is_drawing{              //se sta disegnando
                    is_drawing = true;
                    start = coordinates;
                }
            }else{             //pulsante non premuto
                if is_drawing{   //ma lo stato dice che sta disegnando
                    is_drawing = false; //aggiorno lo stato al valore corretto
                    end = coordinates;
                    //println!("segmento tracciato, coordinate salvate: {}, {}, {}, {}", start.0, start.1, end.0, end.1);
                    if horizontal_check2(start, end, w){
                        //println!("ho passato il check");
                        sender.send(true).unwrap();
                        break;
                    }
                }
            }
        }
    });
    receiver
}
//verifico che i punti start ed end siano grossomodo allineati verticalmente &&
//che il segmento tracciato sia almeno il 90% dell'altezza dello schermo
fn vertical_check(start: (i32, i32), end: (i32, i32), height: f64) -> bool {
    //println!("sono in vertical check");
    let tolerance = 50;
    //let b1 = start.0 >=  end.0 - tolerance && start.0 <= end.0 + tolerance;
    //let b2 = (end.1 - start.1).abs() as f64 > 0.9 * height;
    //println!("{}, {}", b1, b2);
    (start.0 >=  end.0 - tolerance && start.0 <= end.0 + tolerance) && (end.1 - start.1).abs() as f64 > 0.9 * height
}

//verifico che i punti start ed end siano grossomodo allineati orizzontalmente &&
//che il segmento tracciato sia almeno il 90% della lunghezza dello schermo
fn horizontal_check(start: (i32, i32), end: (i32, i32), width: f64) -> bool {
    let tolerance = 50;
    //let b1 = start.1 >=  end.1 - tolerance && start.1 <= end.1 + tolerance;
    //let b2 = (start.0 - end.0).abs() as f64 > 0.90 * width;
    //println!("{}, {}", b1, b2);
    (start.1 >=  end.1 - tolerance && start.1 <= end.1 + tolerance) && (start.0 - end.0).abs() as f64 > 0.90 * width
}

fn horizontal_check2(start: (i32, i32), end: (i32, i32), width: f64) -> bool {
    let tolerance = 50;
    (start.1 >=  end.1 - tolerance && start.1 <= end.1 + tolerance) && (start.0 - end.0).abs() as f64 > 0.90 * width
}