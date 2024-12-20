use std::fs::{self, File}; // modulo per il filesystem
use std::io::{self, BufRead, Write}; // modulo per l'I/O
use std::path::{Path};
use std::ptr::read;
use sysinfo::{System, Disks, Pid, ProcessesToUpdate, ProcessRefreshKind};
use chrono::Utc;
use std::time::{Instant, Duration};
use std::{env, thread};
use eframe::egui::debug_text::print;


/***
copy_dir: copia nel percorso di destinazione il contenuto del percorso sorgente (funzione wrapper di copy_dir_recursive)
    src: stringa del percorso sorgente
    dst: stringa del percorso di destinazione (se la destinazione è una cartella non vuota viene creata una cartella)
    extensions: vettore di stringhe delle estensioni dei file da copiare, se è vuoto vengono copiati tutti i file
    -> restituisce: Ok(dimensione) se l'operazione è andata a buon fine
***/
pub fn copy_dir(src: &str, dst: &str, extensions: Vec<String>) -> io::Result<u64> {
    println!("Copying {} to {}", src, dst);
    let start = Instant::now();

    let src_path = Path::new(src);
    let dst_path = Path::new(dst);

    if !src_path.exists() {
        println!("Percorso per l'origine del backup ('{}') non trovato, backup annullato.", src);
        return Ok(0);
        //return Err(io::Error::new(io::ErrorKind::NotFound, "Sorgente non trovata"));
    }

    // Contare il numero totale di file e la dimensione totale dei file da copiare
    let (total_files, total_size) = count_files_and_size(src_path, &extensions)?;

    //println!("total size: {}", total_size);

    // Verifica se c'è abbastanza spazio nella destinazione
    let available_space = get_available_space(dst_path)?;
    if available_space < total_size {
        println!("Spazio insufficiente nella destinazione. Backup annullato.");
        return Err(io::Error::new(io::ErrorKind::Other, "Spazio insufficiente nella destinazione"));
    }

    // Verifica se il percorso di destinazione esiste e non è vuoto
    if dst_path.exists() {
        if fs::read_dir(dst_path)?.count() > 0 {
            // Se non è vuoto, crea una nuova cartella "backup" dentro la cartella di destinazione
            let backup_path = dst_path.join("backup");
            println!("Il percorso di destinazione non è vuoto. Creazione della cartella 'backup'.");
            fs::create_dir_all(&backup_path)?;  // Crea la cartella "backup"
        }
    } else {
        // Se il percorso di destinazione non esiste, crea la cartella
        println!("Creazione della cartella di destinazione in corso.");
        fs::create_dir(dst_path)?;
    }

    println!("Inizio del backup");

    // Aggiorna il percorso di destinazione per utilizzare la cartella "backup"
    let backup_path = if dst_path.join("backup").exists() {
        dst_path.join("backup")
    } else {
        dst_path.to_path_buf()
    };

    let mut copied_files = 0;  // Contatore dei file copiati
    let mut copied_size = 0;   // Variabile per accumulare la dimensione dei file copiati

    // Chiamata alla funzione ricorsiva
    copy_dir_recursive(src_path, &backup_path, total_files, &mut copied_files, &mut copied_size, &extensions)?;

    let duration = start.elapsed();

    if let Err(e) = save_log(backup_path.to_str().unwrap(), duration, copied_size) {
        eprintln!("Errore durante il salvataggio del log: {}", e);
    }

    println!("\nBackup terminato con successo.");  // Stampa nuova riga dopo il completamento


    Ok(copied_size)
}

fn save_log(path: &str, duration: Duration, size_in_bytes: u64) -> io::Result<()> {
    // Crea il percorso per il file log.txt
    let log_path = Path::new(path).join("backup_log.txt");

    // Apre (o crea) il file log.txt in modalità scrittura
    let mut file = File::create(log_path)?;

    // Converte la durata in secondi e la quantità in MB
    let duration_secs = duration.as_secs_f64();
    let size_in_mb = size_in_bytes as f64 / (1024.0 * 1024.0);

    // Scrive nel file le informazioni
    writeln!(file, "Dimensione totale del backup: {:.2} MB ({:.2} Bytes)", size_in_mb, size_in_bytes)?;
    writeln!(file, "Tempo di CPU impiegato: {:.2} secondi", duration_secs)?;

    Ok(())
}

/***
copy_dir_recursive: funzione di copia ricorsiva
    src_path: Path della sorgente
    dst_path: Path della destinazione
    total_files: numero totale dei file da copiare
    copied_files: numero totale dei file copiati finora (utile per la percentuale di avanzamento)
    copied_size: dimensione totale dei file copiati finora
    extensions: vettore di stringhe delle estensioni dei file da copiare, se è vuoto vengono copiati tutti i file
    -> restituisce: Ok(()) se l'operazione è andata a buon fine
***/
fn copy_dir_recursive(src_path: &Path, dst_path: &Path, total_files: usize, copied_files: &mut usize, copied_size: &mut u64, extensions: &Vec<String>) -> io::Result<()> {
    if !dst_path.exists() {         // se il percorso di destinazione non esiste, crea la cartella
        fs::create_dir(dst_path)?;
    }

    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst_path.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path, total_files, copied_files, copied_size, extensions)?;  // Chiamata ricorsiva per le directory
        } else {
            // Se il vettore di estensioni non è vuoto, copia solo i file che corrispondono alle estensioni
            if extensions.is_empty() || check_extension(&path, extensions) {
                let file_size = path.metadata()?.len();
                fs::copy(&path, &dest_path)?;  // Copia i file
                *copied_files += 1;  // Incrementa il numero di file copiati
                *copied_size += file_size; // Incrementa la dimensione totale dei file copiati
                print_progress(*copied_files, total_files);  // Stampa l'avanzamento sulla stessa riga
            }
        }
    }

    Ok(())
}

/***
check_extension: funzione di controllo dell'estensione del file
    path: Path del file da controllare
    extensions: vettore di stringhe delle estensioni
    -> restituisce: true se l'estensione corrisponde a una delle estensioni specificate, altrimenti false
***/
fn check_extension(path: &Path, extensions: &Vec<String>) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return extensions.iter().any(|e| e == ext_str);
        }
    }
    false
}

/***
print_progress: funzione di stampa dell'avanzamento del backup
    copied: numero totale dei file copiati al momento corrente
    total: numero totale dei file da copiare
***/
fn print_progress(copied: usize, total: usize) {
    let percentage = (copied as f64 / total as f64) * 100.0;  // Calcola la percentuale
    print!("\rAvanzamento: {:.2}%", percentage);  // Stampa la percentuale sulla stessa riga
    io::stdout().flush().unwrap();  // Forza l'output del buffer
}

/***
count_files_and_size: funzione per contare il numero totale di file e la dimensione totale dei file da copiare
    path: Path della sorgente
    extensions: vettore di stringhe delle estensioni
    -> restituisce: un tuple con il numero totale di file e la dimensione totale in byte
***/
fn count_files_and_size(path: &Path, extensions: &Vec<String>) -> io::Result<(usize, u64)> {
    let mut count = 0;
    let mut size = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            let (sub_count, sub_size) = count_files_and_size(&entry_path, extensions)?;  // Chiamata ricorsiva per le sottodirectory
            count += sub_count;
            size += sub_size;
        } else if extensions.is_empty() || check_extension(&entry_path, extensions) {
            count += 1;  // Incrementa il conteggio solo per i file con le estensioni richieste
            size += entry.metadata()?.len();  // Aggiungi la dimensione del file
        }
    }

    Ok((count, size))
}

/***
get_available_space: funzione per ottenere lo spazio disponibile sul disco di destinazione
    path: Path della destinazione
    -> restituisce: la dimensione disponibile in byte
***/
fn get_available_space(path: &Path) -> io::Result<u64> {
    let mut sys = System::new_all();
    sys.refresh_all();
    for disk in Disks::new_with_refreshed_list().list() {
        if path.starts_with(disk.mount_point()) {
            return Ok(disk.available_space());
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Disco non trovato"))
}

/***
get_src_path: funzione per leggere il path di origine dal file di configurazione
    filename: nome del file di configurazione
    -> restituisce: il percorso di origine come stringa, None in caso di errore.
***/
pub fn get_src_path(filename: &str) -> Option<String> {

    let file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => {
            println!("File di configurazione non trovato. Nessun file al percorso '{}'.", filename);
            return None;
        }
    };
    println!("File di configurazione trovato, lettura dell'origine del backup in corso.");
    let mut reader = io::BufReader::new(file);
    let mut first_line = String::new();

    if reader.read_line(&mut first_line).is_err() || first_line.trim().is_empty() {
        println!("Impossibile leggere l'origine del backup dal file di configurazione.");
        return None;
    }

    let src = first_line.trim_end().to_string();
    let src_path = Path::new(&src);

    if !src_path.exists() {
        println!("Percorso per l'origine del backup ('{}') non trovato.", src);
        return None;
    }

    println!("Sorgente del backup: '{}'.", src);
    Some(src)
}

/***
get_extensions: funzione per leggere le estensioni dalla seconda riga in poi del file di configurazione
    filename: nome del file di configurazione
    -> restituisce: un vettore di stringhe delle estensioni lette
***/
pub fn get_extensions(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    println!("Lettura delle estensioni dal file di configurazione.");
    let reader = io::BufReader::new(file);
    let mut extensions = Vec::new();

    for line in reader.lines().skip(1) {  // Salta la prima riga
        if let Ok(line) = line {
            // Suddivide la riga in base a spazi o virgole e filtra le stringhe vuote
            let parts: Vec<String> = line
                .split_whitespace() // o `split(',')` se vogliamo usare la virgola
                .filter_map(|s| {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        Some(trimmed.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            extensions.extend(parts); // Aggiunge le parti al vettore delle estensioni
        }
    }


    let mut backup_mod = "tutti i file".to_string();
    if !extensions.is_empty() {
        backup_mod = "solo file di determinati tipi".to_string();
    }
    println!("Modalità di backup: {}", backup_mod);

    if !extensions.is_empty() {
        let extensions_list = extensions.iter()
            .map(|ext| format!("'{}'", ext))
            .collect::<Vec<String>>()
            .join(", ");
        println!("(estensioni: {})", extensions_list);
    }

    Ok(extensions)
}


/***
get_usb_path: funzione per ottenere il path del disco rimovibile con più spazio disponibile
    -> restituisce: una stringa con il percorso del disco rimovibile trovato se esiste, altrimenti None
***/
pub fn get_usb_path() -> Option<String> {
    let mut sys = System::new_all(); // istanza del sistema
    sys.refresh_all();
    let disks = Disks::new_with_refreshed_list();

    let mut max_free_space = 0;
    let mut usb_path = None;

    for disk in disks.list() { // itera sui dischi disponibili
        if disk.is_removable() {
            let free_space = disk.available_space();
            if free_space > max_free_space { // confronta lo spazio libero
                max_free_space = free_space; // aggiorna il massimo spazio libero
                usb_path = Some(disk.mount_point().to_str()?.to_string()); // aggiorna il percorso
            }
        }
    }

    if usb_path.is_some() {
        println!("Disco esterno trovato, destinazione del backup: '{}'.", usb_path.clone().unwrap().to_string());
    }
    else {
        println!("Nessun disco esterno trovato.");
    }

    usb_path
}

/***
generate_backup_name: funzione per generare un nome per il backup
    src: stringa del percorso sorgente
    dest: stringa del percorso di destinazione
    -> restituisce: una stringa con il nome del backup generato
***/
pub fn generate_backup_name(src: &str, dest: &str) -> String {
    // Split the `dest` string by "\\" and collect the parts into a vector
    let parts: Vec<&str> = src.split("\\").collect();

    // Get the last part of the vector
    if let Some(last_word) = parts.last() {
        // Get the current timestamp in the desired format
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

        // Create the backup name
        return format!("{}{}_backup_{}", dest, last_word, timestamp);
    }

    // Return an empty string if `dest` does not contain any parts
    String::new()
}

pub fn log_cpu_usage() {

    let filename = "log.txt";
    let mut sys = System::new_all(); // Crea un'istanza del sistema che raccoglie le informazioni
    let pid = std::process::id(); // Ottieni l'ID del processo corrente

    // quando il programma si avvia un bootstrap il path relativo cambia (non è più la root del progetto ma la root del sistema).
    let exe = env::current_exe(); //ottengo la root del progetto a partire dalla posizione del file eseguibile (debug/release: stesso effetto (3 parent))
    let binding = exe.unwrap();
    let project_root = binding.parent().unwrap().parent().unwrap().parent();

    let mut log_file = File::create(project_root.unwrap().join(filename)).unwrap(); // Crea o apre il file di log

    //let mut log_file = File::create("log.txt").unwrap(); // Crea o apre il file di log //(funziona non in bootstrap)

    let mut elapsed_time =-1;
    let mut sum = 0.0;
    let mut average_cpu_usage;
    let cpus = num_cpus::get();
    let tot_sec = 2;

    // dalla documentazione: Wait because CPU usage is based on diff.
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

    loop {
        //dalla documentazione: To start to have accurate CPU usage, a process needs to be refreshed twice because CPU usage computation is based on time diff
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new().with_cpu()
        );

        if let Some(process) = sys.process(Pid::from_u32(pid)) {
            //dalla documentazione: " process.cpu_usage() Returns the total CPU usage (in %).
            // Notice that it might be bigger than 100 if run on a multi-core machine.
            // If you want a value between 0% and 100%, divide the returned value by the number of CPUs.

            sum += process.cpu_usage()/cpus as f32;
            elapsed_time+=1;

            if elapsed_time >= tot_sec { //sleep ad ogni secondo, reset contatore e log ogni 120
                average_cpu_usage = sum / elapsed_time as f32;
                sum=0.0;
                elapsed_time=-1;

                // Scrive la data e ora corrente nel file di log
                let timestamp = chrono::offset::Local::now().to_string();
                log_file
                    .write_all(timestamp.as_bytes())
                    .expect("Scrittura log fallita");
                log_file.write_all(b"\n").expect("Scrittura log fallita");
                // Scrive l'utilizzo della CPU del processo
                let log_entry = format!("CPU usage in the last {} seconds: {:05.2}%\n", tot_sec, average_cpu_usage);
                log_file
                    .write_all(log_entry.as_bytes())
                    .expect("Scrittura log fallita");

                log_file.write_all(b"\n").expect("Scrittura log fallita");
            }

        } else {
            println!("Nessun processo trovato con PID = {}", pid);
            break; // Esci dal ciclo se il processo non è più disponibile
        }

        thread::sleep(Duration::from_secs(1)); // Pausa di 1 secondo per ogni loop
    }
}




