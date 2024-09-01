# Gruppo 5 - Progetto 2.1: Back-up di emergenza

## Introduzione

Questo progetto, sviluppato per il corso di Programmazione di Sistema al Politecnico di Torino, mira a creare un'applicazione di backup di emergenza operativa anche in assenza di un monitor funzionante. L'app è scritta in Rust e utilizza SLint per l'interfaccia grafica (GUI).

## Scopo dell'Applicazione

L'applicazione consente di effettuare backup di emergenza in situazioni in cui il monitor del PC è guasto. L'utente può avviare e confermare il backup utilizzando movimenti specifici del mouse, garantendo un feedback acustico per confermare le azioni eseguite

## Funzionalità

### Configurazione del Backup

Al primo avvio, l'applicazione mostra una schermata di configurazione dove è possibile:

- **Selezionare le Cartelle**:
  - **Sorgente**: Cartella da cui eseguire il backup.
  - **Destinazione**: Cartella o unità di destinazione del backup.

- **Scegliere la Modalità di Backup**:
  - **Folder**: Effettua il backup dell'intera cartella sorgente.
  - **Single Files**: Effettua il backup dei file corrispondenti a specifiche estensioni fornite dall'utente attraverso un apposito campo di testo.

Le informazioni di configurazione vengono salvate in un file di testo (`configuration.txt`), garantendo che le preferenze siano mantenute tra i diversi avvii dell'applicazione.

### Esecuzione del Backup

#### Avvio del Backup

Per avviare il backup, l'utente deve:

1. Tracciare un rettangolo lungo i bordi dello schermo tenendo premuto il tasto sinistro del mouse.
2. All'inserimento corretto del comando, l'applicazione emette un segnale acustico ("bip") e mostra una finestra di conferma.

#### Conferma del Backup

Per confermare l'esecuzione del backup, l'utente deve:

1. Tracciare un segno di "meno" (-) da un lato all'altro dello schermo.
2. Anche in questo caso, viene emesso un "bip" per indicare che il comando è stato riconosciuto.

Se l'utente desidera annullare il backup, può farlo tracciando un comando diverso o tramite l'interfaccia grafica.

#### Completamento del Backup

Al termine del backup, l'applicazione:

- Emette tre "bip" consecutivi per indicare che l'operazione è stata completata.
- Ritorna in attesa di un nuovo comando di backup.

### Modalità di Funzionamento

L'app è compatibile con Windows, Linux e macOS e supporta sia la modalità chiara che scura, adattandosi alle impostazioni del sistema operativo. All'avvio del sistema, l'app "Group5" si avvia automaticamente. Alla prima esecuzione, configura l'avvio automatico e nasconde la finestra del terminale quando necessario: su Windows tramite `#![windows_subsystem = "windows"]` e su macOS tramite uno script dedicato.

### Logging

L'applicazione registra due tipi di log:

- **Consumo CPU**: Ogni 2 minuti, viene salvato il consumo di CPU dell'applicazione nel file `log.txt`.
- **Dettagli del Backup**: Al termine di ogni backup, l'applicazione scrive un file `backup_log.txt` nella cartella di destinazione, contenente:
  - La quantità di byte copiati.
  - Il tempo impiegato per eseguire il backup.
 
### Ottimizzazione delle Prestazioni

L'applicazione è progettata per minimizzare il consumo di CPU:

- Uso di un Singolo Thread: Utilizza un solo thread per il backup, poiché la copia dei file raggiunge già la massima velocità del disco con un solo thread. L'aggiunta di thread supplementari non accelererebbe il processo e potrebbe rallentare le operazioni a causa della concorrenza per le risorse del disco.
- Efficienza nei Comandi del Mouse: Durante il riconoscimento dei movimenti del mouse, l'applicazione inserisce delle pause (sleep) per ridurre l'uso non necessario della CPU.

### Autori

- Paolo Cagliero - s324194
- Luciana Galliani - s331469 
- Alessandro Garzaro - s315129

***

## Screenshot

### *Schermata di configurazione*  

![Schermata di configurazione](/readme_assets/configuration.png)

### *Conferma comando di backup*

![Conferma comando backup](/readme_assets/confirm_backup.png)

### *Backup completato*

![Backup completato](/readme_assets/backup_success.png)

### *Errore Backup*

![Errore backup](/readme_assets/backup_error.png)
