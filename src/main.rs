//Nasconde la console del terminale in Windows
#![windows_subsystem = "windows"]

mod backup;

slint::include_modules!();
use std::{env, io, thread};
use std::cell::{Ref, RefCell};
use auto_launch::{AutoLaunchBuilder};
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};  //La import di command serve per MacOS
use std::rc::Rc;
use std::sync::mpsc;
use slint::{ SharedString};
use rfd::FileDialog;

enum MainThreadMessage {
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

    //Leggo il file di configurazione. Se è presente, carico i dati nella GUI, altrimenti inizializzo la GUI vuota
    let content: Vec<String>;
    if Path::new("configuration.txt").exists() {
        content = read_to_string("configuration.txt").unwrap().lines().map(String::from).collect();
        if content.len() == 2 {
            let options: Vec<&str>;
            options = content[1].split(";").collect();

            if options[0] == "F" {
                ui.set_selected_backup_mode(SharedString::from("Folder"));
                ui.set_formatted_file_formats(SharedString::from(""));
            }
            else {
                ui.set_selected_backup_mode(SharedString::from("Single files"));
                ui.set_formatted_file_formats(SharedString::from(options[0]));
                for e in options[0].split(",") {
                    file_formats.borrow_mut().push(e.to_string());
                }
            }
            ui.set_source_folder(SharedString::from(options[1]));
            ui.set_destination_folder(SharedString::from(options[2]));
        }
        else {
            //File di configurazione non valido
            ui.set_selected_backup_mode(SharedString::from("Folder"));
            ui.set_formatted_file_formats(SharedString::from(""));
            ui.set_source_folder(SharedString::from(""));
            ui.set_destination_folder(SharedString::from(""));
        }
    }
    else {
        //File di configurazione non trovato
        ui.set_selected_backup_mode(SharedString::from("Folder"));
        ui.set_formatted_file_formats(SharedString::from(""));
        ui.set_source_folder(SharedString::from(""));
        ui.set_destination_folder(SharedString::from(""));
    }

    //Gestione dei callback
    //ui
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

    ui.on_folder_selected({
        let ui_handle = ui.as_weak();
        let file_formats = Rc::clone(&file_formats);    //Creo un riferimento a file_formats. In questo modo, posso modificarlo all'interno della closure subito sotto e averlo disponibile anche nella closure di on_save_button_clicked

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                // Resetta file_formats e aggiorna l'interfaccia utente


                file_formats.borrow_mut().clear();
                ui.set_formatted_file_formats(SharedString::from("")); // Pulire la visualizzazione dei formati
            }
        }
    });

    ui.on_save_button_clicked({
        let ui_handle3 = ui.as_weak();
        let file_formats = Rc::clone(&file_formats);
        move || {
            if let Some(ui) = ui_handle3.upgrade() { // la necessità di fare l'upgrade era necessria per aver
                // il diritto di deallocare  uno spazio di memoria

                //salvo le stringhe del file, path sorgente e path destinazione
                let mut formats = file_formats.borrow().join(",");

                let source = ui.get_source_folder().to_string();
                let destination = ui.get_destination_folder().to_string();

                if !source.is_empty()&& !destination.is_empty() {

                    if formats.is_empty() {
                        formats = "F".to_string();
                    }
                    ui.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra


                    if let Err(e) = create_file_configuration(&formats, &source, &destination) {
                        eprintln!("Error creating configuration file: {}", e);
                    } else {
                        //Creo questo vettore di stringhe contenente le informazioni di configurazione del backup, che viene passato alla funzione che si occupa di effettuare il backup
                        let options: Vec<String> = Vec::from([formats.to_string(), source.to_string(), destination.to_string()]);
                        backup::start_backup(tx.clone(), tx_close.clone(), options);
                    }
                }

            }
        }

    });

    ui.on_quit_button_clicked({
        let ui_handle3 = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle3.upgrade() {
                ui.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
                exit(0);
            }
        }
    });

    ui.window().on_close_requested({
        move || {
            exit(0);
        }
    });

    //confirm_mess
    confirm_mess.on_abort_button_clicked({    //Quando clicco su annulla, in automatico viene annullato il backup, siccome sto inserendo un comando diverso da quello di conferma
        let ui_handle = confirm_mess.as_weak();
        move || {
            if let Some(confirm_mess) = ui_handle.upgrade() {
                confirm_mess.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
            }
        }
    });

    //backup_compl_mess
    backup_compl_mess.on_close_button_clicked({
        let ui_handle = backup_compl_mess.as_weak();
        move || {
            if let Some(backup_compl_mess) = ui_handle.upgrade() {
                backup_compl_mess.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
            }
        }
    });

    //backup_err_mess
    backup_err_mess.on_close_button_clicked({
        let ui_handle = backup_err_mess.as_weak();
        move || {
            if let Some(backup_err_mess) = ui_handle.upgrade() {
                backup_err_mess.hide().expect("Impossibile nascondere la finestra"); // Nascondi/Chiudi la finestra
            }
        }
    });

    //Thread che, quando riceve un messaggio (rx_close), chiude la finestra di confirm_mess.
    //Ho bisogno di usare un thread diverso da quello principale siccome, quando una finestra della GUI è aperta, il thread principale è impegnato a gestire la GUI e non riuscirebbe a ricevere il messaggio
    let confirm_mess_weak = confirm_mess.as_weak();
    thread::spawn(move || {
        loop {
            rx_close.recv().unwrap();

            let confirm_mess_weak = confirm_mess_weak.clone();
            slint::invoke_from_event_loop({
                move || {
                    if let Some(window) = confirm_mess_weak.upgrade() {
                        window.hide().expect("Impossibile nascondere la finestra");
                    }
                }
            }).unwrap();
        }
    });

    let _ = ui.run();

    //Quando la funzione di backup ha bisogno che venga mostrata una GUI, invia un messaggio al thread principale
    //Il thread principale riceve il messaggio e visualizza la finestra corretta
    loop {
        if let Ok(msg) = rx.recv() {
            match msg {
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

fn create_file_configuration(file_formats: &String, path_source: &String, path_destination: &String) -> io::Result<()> {
    let configuration_file = "configuration.txt";
    let mut file = File::create(configuration_file)?;



    // Scrivere l'intestazione
    writeln!(file, "type;source;destination")?;

    // Scrivere i dati nel file
    writeln!(file, "{};{};{}", file_formats, path_source, path_destination)?;

    println!("Configuration file written successfully.");
    Ok(())
}