use std::process::{Command, Stdio};
use std::io::{self, BufRead, Write};
use std::thread;

fn main() {
    // Crea il processo figlio
    #[cfg(target_os = "windows")]
    let mut child = Command::new("./src/windowsManager.exe")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute child process");

    #[cfg(not(target_os = "windows"))]
    let mut child = Command::new("./src/windowsManager")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute child process");

    // Se il child ha un stdout, leggilo in un thread separato
    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        thread::spawn(move || {
            for line in reader.lines() {
                println!("Figlio: {}", line.expect("Could not read line"));
            }
        });
    }

    // Attendi l'input dell'utente per terminare il processo figlio
    println!("Premi Invio per terminare il processo figlio...");
    let _ = io::stdin().read_line(&mut String::new());

    // Termina il processo figlio
    child.kill().expect("failed to kill child process");
    println!("Processo figlio terminato.");
}
