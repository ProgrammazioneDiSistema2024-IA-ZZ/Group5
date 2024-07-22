use std::fs;
use std::fs::{read_to_string};
use std::path::Path;
use device_query::{DeviceQuery, DeviceState, MouseState};
use std::time::Duration;
use rodio::{source::SineWave, OutputStream, Sink, Source};
use copy_dir::copy_dir;

fn main() {
    let device_state = DeviceState::new();

    let mut is_drawing = false;
    let mut start_position = (0, 0);
    let mut end_position: (i32, i32);

    let mut options: Vec<&str> = Vec::new();

    //Leggo il file di configurazione per capire il percorso sorgente, il percorso destinazione e il tipo di backup.
    //Se il file di configurazione è assente o è vuoto, devo configurare il programma
    let content: Vec<String> = read_to_string("configuration.txt").unwrap().lines().map(String::from).collect();
    //TODO: se il file non esiste?
    if content.len() == 2 {
        options = content[1].split(";").collect();
    }
    else {
        //TODO: configurare il programma
    }


    //Vettore di 4 elementi che rappresentano i lati di un rettangolo. Se il primo elemento è V (lato verticale), il secondo deve essere H (lato orizzontale), poi V e infine H. Altrimenti, si potrebbe avere H, V, H, V
    let mut sides: Vec<char> = Vec::with_capacity(4);
    //Questa variabile indica se ho già riprodotto il beep di conferma del primo dei due comandi
    let mut sound_played = false;

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
                //In questo modo, c'è meno rischio che venga disegnato un rettangolo "accidentalmente", durante le normali operazioni al PC.

                if sides.len() == 0 {
                    if is_vertical(start_position, end_position) {
                        sides.push('V');
                    }
                    else {
                        if is_horizontal(start_position, end_position) {
                            sides.push('H');
                        }
                    }
                }
                else {
                    if (sides[sides.len() - 1] == 'V' && is_horizontal(start_position, end_position)) || (sides[sides.len() - 1] == 'H' && is_vertical(start_position, end_position) || sound_played) {
                        if is_horizontal(start_position, end_position) && sides.len() < 4 {
                            sides.push('H');
                        }
                        else {
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
                                //TODO: Mostrare a video una schermata di conferma

                                sound_played = true;
                            }
                            else {
                                if is_horizontal(start_position, end_position) {
                                    //A questo punto, effettuo il backup

                                    //Riproduco un suono di conferma anche in questo caso
                                    play_sound();

                                    if options[0] == "F" {
                                        //Effettuo il backup di una cartella. Per prima cosa, elimino la cartella di destinazione (la funzione copy_dir ritorna errore se la cartella di destinazione esiste), e successivamente copio il contenuto della cartella sorgente in quella di destinazione
                                        if Path::new(options[2]).exists() {
                                            fs::remove_dir_all(options[2]).expect("Non sono riuscito a rimuovere la cartella");

                                            println!("Cartella rimossa");
                                        }
                                        copy_dir(options[1], options[2]).expect("Backup fallito");
                                        println!("Backup completo");
                                    }
                                    else {
                                        //TODO: capire quali sono i tipi di backup
                                    }
                                }
                            }
                        }
                    }
                    else {  //Non ho né una linea orizzontale né una verticale, quindi resetto il vettore
                        sides = Vec::with_capacity(4);
                    }
                }
            }
        }
    }
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