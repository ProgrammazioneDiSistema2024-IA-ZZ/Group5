//Nasconde la console del terminale in Windows
//#![windows_subsystem = "windows"]

slint::include_modules!();
use std::{env, fs, io, thread};
use std::cell::{Ref, RefCell};
use auto_launch::{AutoLaunchBuilder};
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use device_query::{DeviceQuery, DeviceState, MouseState};
use std::time::{Duration};
use rodio::{source::SineWave, OutputStream, Sink, Source};
use copy_dir::copy_dir;
use sysinfo::System;
use chrono;
use cpu_time::ProcessTime;
use fs_extra::dir::get_size;
use glob::glob;
use rdev::{display_size};
use slint::{ModelRc, SharedString};
use rfd::FileDialog;

enum MainThreadMessage {
    ShowErrorMessage,
    ShowConfirmMessage,
    ShowBackupCompleteMessage,
    ShowBackupErrorMessage
}

fn main() {

    /*
Queste due righe vengono utilizzate per ottenere il percorso
dell'eseguibile corrente e il percorso della directory che lo contiene
*/
    let exe = env::current_exe().unwrap(); // exe contiene il percorso completo dell'eseguibile corrente
    let wd = exe.parent().unwrap(); //wd contiene il percorso della directory che contiene l'eseguibile corrente
    let app_path = wd.join("Group5");


    //Inizializzo le schermate della GUI di cui ho bisogno
    let ui = AppWindow::new().unwrap();
    let err_mess = ErrorMessage::new().unwrap();
    let confirm_mess = ConfirmMessage::new().unwrap();
    let backup_compl_mess = BackupCompletedMessage::new().unwrap();
    let backup_err_mess = BackupErrorMessage::new().unwrap();
    let file_formats: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new())); //Dichiaro file_formats come Rc così che possa essere condiviso tra più closure

    let (tx, rx) = mpsc::channel();
    let (tx_close, rx_close) = mpsc::channel();


    #[cfg(not(target_os = "macos"))] //questo codice sarà eseguito solo su Windows e Linux
    {
        let auto = AutoLaunchBuilder::new()
            .set_app_name("Group5")
            .set_app_path(&app_path.to_str().unwrap())  //Imposta il percorso dell'applicazione che deve essere avviata automaticamente
            .set_use_launch_agent(false)
            .build()
            .unwrap();


        auto.enable().unwrap();
        println!("Autostart enabled: {}", auto.is_enabled().unwrap());
    }

    #[cfg(target_os = "macos")] //questo codice sarà eseguito solo su macos
    {
        let _ = AutoLaunchBuilder::new()
            .set_app_name("Group5")
            .set_app_path(&app_path.to_str().unwrap())
            .set_use_launch_agent(false) //non utilizza un "launch agent" per l'avvio automatico
            .build()
            .unwrap().enable();
        //per nascondere la finestra frontale dell'applicazione Terminale, rendendola invisibile all'utente

        Command::new("osascript")
            .arg("-e")
            .arg("tell application \"Terminal\" to set visible of front window to false")
            .output()
            .expect("Failed to hide terminal");
    }

    // Gestione dei callback
    ui.on_add_file_formats({
        let ui_handle = ui.as_weak();
        let file_formats = Rc::clone(&file_formats);    //Creo un riferimento a file_formats. In questo modo, posso modificarlo all'interno della closure subito sotto e averlo disponibile anche nella closure di on_save_button_clicked
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let new_format = ui.get_file_format_input().to_string();

                if !new_format.is_empty() && !file_formats.borrow().contains(&new_format) {
                    file_formats.borrow_mut().push(new_format.clone());

                    let formats = convert_file_formats(file_formats.borrow());

                    ui.set_formatted_file_formats(SharedString::from(formats));
                }
            }
        }
    });

    ui.on_select_source_folder_clicked( {
        let ui_handle = ui.as_weak();
        move || {
            if let Some(folder_path) = FileDialog::new().pick_folder() {
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_source_folder(SharedString::from(folder_path.display().to_string()));
                }
            }
        }
    });

    ui.on_select_destination_folder_clicked( {
        let ui_handle = ui.as_weak();
        move || {
            if let Some(folder_path) = FileDialog::new().pick_folder() {
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_destination_folder(SharedString::from(folder_path.display().to_string()));
                }
            }
        }
    });

    ui.on_save_button_clicked({
        let ui_handle3 = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle3.upgrade() { // la necessità di fare l'upgrade era necessria per aver
                // il diritto di deallocare  uno spazio di memoria
                ui.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
                start_backup(tx.clone(), tx_close.clone());
            }
        }
    });
    ui.on_quit_button_clicked({
        let ui_handle3 = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle3.upgrade() {
                ui.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
                //TODO: quando si schiaccia su abort il programma rimane in esecuzione (se non è voluto, basta mettere exit(0); al posto di questo commento)
            }
        }
    });

    //Gestisco gli eventi delle GUI
    err_mess.on_exit_button_clicked({
        let ui_handle = err_mess.as_weak();
        move || {
            if let Some(err_mess) = ui_handle.upgrade() {
                err_mess.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
                exit(1);
            }
        }
    });

    err_mess.window().on_close_requested({
        move || {
            exit(1);
        }
    });

    confirm_mess.on_abort_button_clicked({    //Quando clicco su annulla, in automatico viene annullato il backup, siccome sto inserendo un comando diverso da quello di conferma
        let ui_handle = confirm_mess.as_weak();
        move || {
            if let Some(confirm_mess) = ui_handle.upgrade() {
                confirm_mess.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
            }
        }
    });

    backup_compl_mess.on_close_button_clicked({
        let ui_handle = backup_compl_mess.as_weak();
        move || {
            if let Some(backup_compl_mess) = ui_handle.upgrade() {
                backup_compl_mess.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
            }
        }
    });

    backup_err_mess.on_close_button_clicked({
        let ui_handle = backup_err_mess.as_weak();
        move || {
            if let Some(backup_err_mess) = ui_handle.upgrade() {
                backup_err_mess.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
            }
        }
    });

    let confirm_mess_weak = confirm_mess.as_weak();
    thread::spawn(move || {
        rx_close.recv().unwrap();

        let confirm_mess_weak = confirm_mess_weak.clone();
        slint::invoke_from_event_loop({
            move || {
                if let Some(window) = confirm_mess_weak.upgrade() {
                    window.hide().expect("Impossibile nascondere la finestra");
                }
            }
        }).unwrap();
    });

    let _ = ui.run();

    loop {
        if let Ok(msg) = rx.recv() {
            match msg {
                MainThreadMessage::ShowErrorMessage => {
                    let _ = err_mess.run();
                }
                MainThreadMessage::ShowConfirmMessage => {
                    let _ = confirm_mess.run();
                }
                MainThreadMessage::ShowBackupCompleteMessage => {
                    let _ = backup_compl_mess.run();
                }
                MainThreadMessage::ShowBackupErrorMessage => {
                    let _ = backup_err_mess.run();
                }
            }
        }
    }
}


//Funzione che prende come input un vettore di formati dei file e li converte in una stringa di tipo "formato1,formato2,..."
fn convert_file_formats(file_formats: Ref<Vec<String>>) -> String {
    let mut formats: String = String::from("");
    for (i, f) in file_formats.iter().enumerate() {
        for c in f.chars() {
            formats.push(c);
        }
        if i != file_formats.len()-1 {
            formats.push(',');
        }
    }
    formats
}


fn start_backup(tx: Sender<MainThreadMessage>, tx_close: Sender<()>) {
    let device_state = DeviceState::new();

    //Vettore di 4 elementi che rappresentano i lati di un rettangolo. Se il primo elemento è V (lato verticale), il secondo deve essere H (lato orizzontale), poi V e infine H. Altrimenti, si potrebbe avere H, V, H, V
    let mut sides: Vec<char> = Vec::with_capacity(4);
    //Questa variabile indica se ho già riprodotto il beep di conferma del primo dei due comandi
    let mut sound_played = false;

    //Leggo il file di configurazione per capire il percorso sorgente, il percorso destinazione e il tipo di backup.
    //Se il file di configurazione è assente o è vuoto, devo configurare il programma
    std::thread::spawn(move || {
        //Thread che si occupa di effettuare il log del consumo di CPU ogni 2 minuti. Il log viene salvato in un file "log.txt" nella stessa cartella del progetto, e i dati della CPU vengono presi tramite il crate sysinfo.
        let mut sys = System::new();
        thread::spawn(move || {
            let mut log_file = File::create("log.txt").unwrap();

            sys.refresh_cpu();  //Senza questo refresh e la relativa sleep, al primo log il consumo di CPU sarebbe sempre 100%
            thread::sleep(Duration::from_secs(10));

            loop {
                sys.refresh_cpu();                  //Aggiorna le informazioni della CPU. Serve per avere dati aggiornati
                log_file.write(chrono::offset::Local::now().to_string().as_bytes()).expect("Scrittura log fallita");
                log_file.write("\n".as_bytes()).expect("Scrittura log fallita");
                for i in 0..sys.cpus().len() {
                    log_file.write(("CPU ".to_owned() + &*i.to_string() + ": " + &*sys.cpus()[i].cpu_usage().to_string() + "%\n").as_bytes()).expect("Scrittura log fallita");
                }
                log_file.write("\n".as_bytes()).expect("Scrittura log fallita");
                thread::sleep(Duration::from_secs(120));
            }
        });

        let mut is_drawing = false;
        let mut start_position = (0, 0);
        let mut end_position: (i32, i32);

        let content: Vec<String>;
        if Path::new("configuration.txt").exists() {
            content = read_to_string("configuration.txt").unwrap().lines().map(String::from).collect();
            if content.len() == 2 {
                let options: Vec<&str>;
                options = content[1].split(";").collect();

                //Prendo le dimensini dello schermo (larghezza e altezza). Mi servono per verificare che il rettangolo sia disegnato lungo i bordi dello schermo
                let (w_temp, h_temp) = display_size().unwrap();
                //Converto le dimensioni dello schermo da u64 a f64
                let w = w_temp as f64;
                let h = h_temp as f64;

                println!("{} {}", w, h);

                loop {
                    let mouse: MouseState = device_state.get_mouse();
                    let position = mouse.coords;

                    if mouse.button_pressed[1] {
                        if !is_drawing {
                            is_drawing = true;
                            start_position = position;
                            println!("Start drawing at {:?}", start_position);
                        }
                    } else {
                        if is_drawing {
                            is_drawing = false;
                            end_position = position;
                            println!("Finished drawing. Start: {:?}, End: {:?}", start_position, end_position);

                            //L'utente schiaccia il tasto sinistro del mouse e disegna un lato del rettangolo. Rilascia il tasto, lo schiaccia di nuovo e disegna il secondo lato del rettangolo. Così via fino a che il rettangolo non è completo
                            //In questo modo, c'è meno rischio che venga disegnato un rettangolo "accidentalmente" durante le normali operazioni al PC.
                            //Il rettangolo deve essere disegnato lungo i bordi dello schermo. In caso di più monitor, si fa riferimento a quello impostato come principale

                            if sides.len() == 0 {
                                //Il secondo controllo mi serve per capire se il rettangolo sia stato tracciato per almeno il 90% della lunghezza/altezza dello schermo
                                if is_vertical(start_position, end_position) && f64::from((end_position.1 - start_position.1).abs()) > 0.9 * h {
                                    sides.push('V');
                                } else {
                                    if is_horizontal(start_position, end_position) && f64::from((end_position.0 - start_position.0).abs()) > 0.9 * w {
                                        sides.push('H');
                                    }
                                }
                            } else {
                                if (sides[sides.len() - 1] == 'V' && (is_horizontal(start_position, end_position) && f64::from((end_position.0 - start_position.0).abs()) > 0.9 * w)) || (sides[sides.len() - 1] == 'H' && (is_vertical(start_position, end_position) && f64::from((end_position.1 - start_position.1).abs()) > 0.9 * h)) || sound_played {
                                    if is_horizontal(start_position, end_position) && sides.len() < 4 {
                                        sides.push('H');
                                    } else {
                                        if sides.len() < 4 {
                                            sides.push('V');
                                        }
                                    }
                                    println!("{:?}", sides);
                                    if sides.len() == 4 {
                                        //In questo caso ho un rettangolo, mi comporto di conseguenza

                                        if !sound_played {
                                            //Riproduco un beep per confermare che il comando è stato acquisito correttamente
                                            play_sound();
                                            tx.send(MainThreadMessage::ShowConfirmMessage).unwrap();

                                            sound_played = true;
                                        } else {
                                            if is_horizontal(start_position, end_position) && f64::from((end_position.0 - start_position.0).abs()) > 0.9 * w {
                                                //A questo punto, effettuo il backup

                                                //Riproduco un suono di conferma anche in questo caso
                                                play_sound();

                                                //Chiudo eventuali finestre aperte
                                                tx_close.send(()).unwrap();

                                                sides.clear();
                                                sound_played = false;

                                                if Path::new(options[1]).exists() {
                                                    //Effettuo il backup. Per prima cosa, elimino la cartella di destinazione (la funzione copy_dir ritorna errore se la cartella di destinazione esiste)
                                                    if Path::new(options[2]).exists() {
                                                        fs::remove_dir_all(options[2]).expect("Non sono riuscito a rimuovere la cartella");

                                                        println!("Cartella rimossa");
                                                    }
                                                    let start_backup = ProcessTime::try_now().expect("Non sono riuscito ad ottenere il tempo del backup");

                                                    if options[0] == "F" {
                                                        //Effettuo il backup di un'intera cartella
                                                        copy_dir(options[1], options[2]).expect("Backup fallito");
                                                    } else {
                                                        //In options[0] ho un elenco di tipi di file separati da virgola (,). Li estraggo e li inserisco in un vettore. Poi, richiamo la funzione copy_files che effettua il backup di tali file
                                                        let ext: Vec<&str> = options[0].split(',').collect();
                                                        if let Err(e) = copy_files(options[1], options[2], &ext) {
                                                            eprintln!("Error copying files: {}", e);
                                                        }
                                                    }

                                                    let cpu_time: Duration = start_backup.try_elapsed().expect("Non sono riuscito ad ottenere il tempo del backup");
                                                    tx.send(MainThreadMessage::ShowBackupCompleteMessage).unwrap();

                                                    let mut backup_log = File::create(options[2].to_owned() + "/backup_log.txt").unwrap();
                                                    backup_log.write((get_size(options[2]).unwrap().to_string() + " bytes\n").as_bytes()).expect("Scrittura file fallita");
                                                    backup_log.write((cpu_time.as_millis().to_string() + " millis\n").as_bytes()).expect("Scrittura file fallita");
                                                } else {
                                                    //Il percorso di sorgente non esiste, ritorno un errore
                                                    tx.send(MainThreadMessage::ShowBackupErrorMessage).unwrap();
                                                }
                                            } else {
                                                //La forma di conferma non è quella che mi aspettavo, perciò annullo il backup
                                                sides.clear();
                                                sound_played = false;
                                            }
                                        }
                                    }
                                } else {
                                    //Non ho né una linea orizzontale né una verticale (oppure ho due linee orizzontali/verticali consecutive, oppure non ho una linea lunga tanto quanto lo schermo), quindi resetto il vettore
                                    sides.clear();
                                }
                            }
                        }
                    }

                    //Per ridurre il consumo di CPU, faccio una sleep di 50ms durante il loop
                    thread::sleep(Duration::from_millis(50));
                }
            }
            else {
                tx.send(MainThreadMessage::ShowErrorMessage).unwrap();
            }
        }
        else {
            tx.send(MainThreadMessage::ShowErrorMessage).unwrap();
        }
    });
}


fn is_vertical(start: (i32, i32), end: (i32, i32)) -> bool {
    start.0 >= end.0-50 && start.0 <= end.0+50
}

fn is_horizontal(start: (i32, i32), end: (i32, i32)) -> bool {
    start.1 >= end.1-50 && start.1<=end.1+50
}

fn play_sound() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    //Onda a 440 Hz (nota A4) per 500 millisecondi
    let source = SineWave::new(440.0).take_duration(Duration::from_millis(500));
    sink.append(source);

    //Sleep mentre il suono viene riprodotto
    std::thread::sleep(Duration::from_secs(1));
    sink.sleep_until_end();
}

fn copy_files(src: &str, dest: &str, extensions: &[&str]) -> io::Result<()> {
    fs::create_dir_all(dest)?;

    for ext in extensions {
        //Tramite Glob, trovo i file con l'estensione desiderata nella cartella sorgente
        let pattern = format!("{}/**/*.{}", src, ext);

        //Itero i file trovati, e li copio nella cartella di destinazione
        for entry in glob(&pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        //TODO: Durante la copia, non viene mantenuto il percorso di cartelle originale, ma i file vengono tutti copiati nella cartella radice -> da discutere: è un bug o una feature?
                        let file_name = path.file_name().unwrap();
                        let dest_path = Path::new(dest).join(file_name);

                        fs::copy(&path, &dest_path)?;
                        println!("Copied {:?} to {:?}", path, dest_path);
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }

    Ok(())
}