/* CARGO.TOML *
[package]
name = "configFile"
version = "0.1.0"
edition = "2021"

[dependencies]
*/





use std::fs;

static LETTURA_FILE: &str = "./src/path_to_backup.txt";

fn lettura_file(){
    let contents = fs::read_to_string(LETTURA_FILE)
        .expect("Should have been able to read the file");
    println!("Il file di configurazione indica la cartella {}", contents);
