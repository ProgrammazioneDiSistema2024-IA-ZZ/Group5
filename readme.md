# Gruppo 5 - Progetto 2.1: Back-up di emergenza

Lo scopo dell'applicazione è quello di permettere di effettuare backup di emergenza nel caso in cui il monitor del PC sia guasto.

L'applicazione si apre in automatico all'avvio del PC, e mostra una schermata di configurazione attraverso la quale è possibile selezionare le cartelle di sorgente e destinazione del backup, oltre che al tipo di backup.  
È possibile scegliere tra due tipologie di backup:
- Folder: effettua il backup dell'intera cartella sorgente;
- Single files: effettua il backup soltanto dei file che corrispondono all'elenco di estensioni fornito. Quando si seleziona questa modalità, viene richiesto all'utente di inserire una o più estensioni di file tramite un apposito campo di testo.

Schiacciando su Save, le informazioni inserite vengono salvate in un file di testo, `configuration.txt`, così che le preferenze vengano mantenute al successivo avvio dell'app.  
La schermata viene chiusa e l'app attende, in background, che venga inserito il comando di backup (il quale consiste nel tracciare un rettangolo lungo i bordi dello schermo, tenendo premuto il tasto sinistro del mouse).

Quando l'applicazione riconosce il corretto inserimento del comando di backup, questa emette un "bip" (così che l'utente abbia un feedback anche in caso il monitor non funzioni) e mostra a schermo una finestra che conferma di aver correttamente inserito il primo comando.  
Dopodichè, se l'utente vuole effettuare il backup, dovrà inserire il comando di conferma (un "meno" che va da un lato all'altro dello schermo, sempre tenendo premuto il tasto sinistro). Anche in questo caso, viene emesso un "bip" così che l'utente sappia che anche questo comando è stato riconosciuto correttamente.  
Se invece l'utente non è interessato ad effettuare il backup e ha attivato il comando per sbaglio, può annullare l'operazione tramite l'interfaccia grafica oppure tracciando sullo schermo un qualsiasi altro comando diverso da quello di conferma.  

A questo punto, viene eseguito il backup secondo la modalità selezionata in fase di configurazione.  

Al termine del backup, vengono emessi tre "bip" consecutivi e l'applicazione ritorna nuovamente in attesa del comando di backup.

L'app funziona sia in modalità chiara che modalità scura (segue le impostazioni del sistema operativo), ed è compatibile sia per Windows che per Linux che per MacOS.
In proposito, su Windows e Linux, il codice configura e abilita l'avvio automatico dell'applicazione "Group5" all'avvio del sistema operativo.
Su macOS, oltre a configurare e abilitare l'avvio automatico, il codice esegue uno script per nascondere la finestra del Terminale, migliorando l'esperienza utente rendendo invisibile la finestra del Terminale che potrebbe altrimenti apparire.
***

Oltre a queste funzionalità di base, l'applicazione si occupa di scrivere, ogni 2 minuti, il consumo di CPU in un file di log, `log.txt`. Inoltre, quando effettua il backup, scrive in un altro file di log, `backup_log.txt`, nella cartella di destinazione la quantità di byte copiati e il tempo impiegato ad effettuare il backup.

L'applicazione cerca di ridurre al minimo il suo consumo di CPU. Per ottenere ciò, è stato scelto di usare un singolo thread per effettuare il backup invece di più thread separati. Siccome la copia dei file avviene alla massima velocità consentita dal disco anche quando si utilizza un solo thread, aggiungere altri thread non avrebbe velocizzato la copia, ma, dal momento più thread avrebbero "combattuto" per le stesse risorse sul disco, questo avrebbe rallentato le operazioni.  
Inoltre, durante la fase di tracciamento del movimento del mouse per riconoscere il comando di backup, sono state inserite delle opportune `sleep` così da non occupare la CPU per più tempo dello stretto necessario.

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