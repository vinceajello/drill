# Analisi: Eliminazione Binari Separati e Polling

## Stato Attuale

L'applicazione **Drill** utilizza attualmente:

### Architettura Corrente
- **1 binario principale** (`drill`): Gestisce la system tray e il main event loop con `tao`/`tray-icon`
- **2 binari separati** per i pannelli UI:
  - `drill-about`: Finestra About Dialog
  - `drill-create`: Finestra Create Tunnel Dialog
- **Polling mechanism**: Il main loop usa `ControlFlow::Poll` per controllare la creazione di nuovi tunnel dai processi esterni

### Problemi Identificati

1. **Complessità di Build**: Tre binari separati aumentano i tempi di compilazione e la dimensione del pacchetto
2. **Comunicazione Inter-Process**: I binari comunicano tramite stdout/stderr (fragile e limitato)
3. **Polling Inefficiente**: Il main loop deve continuamente controllare lo stato, consumando risorse
4. **Gestione Errori**: Difficile gestire errori tra processi separati
5. **Violazione Main Thread su macOS**: I binari separati non rispettano il requisito di macOS che le GUI debbano girare sul main thread

---

## Soluzioni Proposte

### ✅ Soluzione Raccomandata: `iced::daemon` + Multi-Window Support

#### Vantaggi
- **Un solo binario**: Tutta la logica UI in un unico eseguibile
- **Nativo multi-window**: Iced supporta nativamente finestre multiple dal v0.13+
- **Rispetta macOS main thread**: `iced::daemon` gestisce correttamente l'event loop su tutti gli OS
- **No polling**: Event-driven architecture con `Task` e `window::open`/`window::close`
- **Type-safe**: Tutte le finestre condividono lo stesso state type-safe

#### Come Funziona

```rust
use iced::{daemon, window, Task};
use std::collections::BTreeMap;

struct App {
    windows: BTreeMap<window::Id, WindowType>,
    tunnel_manager: Arc<Mutex<TunnelManager>>,
}

enum WindowType {
    TrayMenu,    // Invisibile, solo per system tray
    About,
    CreateTunnel,
}

fn main() -> iced::Result {
    daemon(App::new, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
```

#### Implementazione
1. Convertire `main.rs` da `tao::event_loop` a `iced::daemon`
2. Creare enum `WindowType` per distinguere le finestre
3. Usare `window::open()` per aprire finestre on-demand
4. Usare `window::close_events()` subscription per gestire chiusure
5. Mantenere la system tray con `tray-icon` (compatibile con iced)

---

### 🔄 Soluzione Alternativa: `native-dialog` per Dialog Semplici

#### Applicabile Solo Per
- About Dialog (informazioni statiche)
- Alert/Conferme semplici

#### NON Applicabile Per
- Create Tunnel Dialog (form complesso con validazione)

#### Vantaggi
- Molto leggero
- Usa dialog nativi OS (NSAlert su macOS)
- Non richiede finestre separate

#### Limitazioni
- Solo dialog modali semplici
- Non supporta form complessi
- Non supporta input validazione real-time

#### Esempio

```rust
use native_dialog::{MessageDialog, MessageType};

pub fn show_about_dialog() {
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("About Drill")
        .set_text("Drill v0.1.0\n\nA multi-platform tunnel drilling application")
        .show_alert()
        .unwrap();
}
```

**Conclusione**: Usare solo per About, non per Create Tunnel

---

### ❌ Soluzioni NON Raccomandate

#### 1. Mantenere Binari Separati
**Perché No**:
- Viola il main thread requirement su macOS
- Inefficiente (polling)
- Difficile debug e manutenzione

#### 2. Usare GTK/Qt Bindings
**Perché No**:
- Dipendenze pesanti
- Complessità di packaging su macOS
- Non idiomatico per system tray app

---

## Piano di Implementazione Raccomandato

### Fase 1: Preparazione
- [x] Analisi architettura attuale
- [x] Ricerca soluzioni multipiattaforma
- [ ] Backup del codice attuale

### Fase 2: Migrazione a `iced::daemon`
- [ ] Creare nuova struttura `App` con `BTreeMap<window::Id, WindowType>`
- [ ] Migrare event loop da `tao` a `iced::daemon`
- [ ] Mantenere integrazione `tray-icon` (compatibile)
- [ ] Implementare `window::open()` per About Dialog
- [ ] Implementare `window::open()` per Create Tunnel Dialog

### Fase 3: Rimozione Binari Separati
- [ ] Eliminare `src/panels/bin/about_dialog.rs`
- [ ] Eliminare `src/panels/bin/create_dialog.rs`
- [ ] Aggiornare `Cargo.toml` (rimuovere `[[bin]]` entries)
- [ ] Integrare UI nel main binary

### Fase 4: Eliminazione Polling
- [ ] Rimuovere `ControlFlow::Poll` logic
- [ ] Rimuovere `pending_tunnel` Arc<Mutex>
- [ ] Usare `Task` per comunicazione tra finestre
- [ ] Implementare `window::close_events()` subscription

### Fase 5: Testing
- [ ] Test su macOS (main thread safety)
- [ ] Test su Windows
- [ ] Test su Linux
- [ ] Verificare memory leaks
- [ ] Performance testing

---

## Modifiche Necessarie al Codice

### `Cargo.toml`

```toml
[dependencies]
iced = { version = "0.13", features = ["tokio", "multi-window"] }
tray-icon = "0.19"
native-dialog = "0.9"  # Solo per About

# RIMUOVERE:
# tao = "0.30"

# RIMUOVERE:
# [[bin]]
# name = "drill-about"
# path = "src/panels/bin/about_dialog.rs"
#
# [[bin]]
# name = "drill-create"
# path = "src/panels/bin/create_dialog.rs"
```

### Struttura File Proposta

```
src/
├── main.rs                    # iced::daemon entry point
├── app.rs                     # App state e logic
├── config.rs                  # Invariato
├── logs.rs                    # Invariato
├── tunnels.rs                 # Invariato
├── systemtray.rs             # Adattato per iced
└── windows/
    ├── mod.rs                 # WindowType enum
    ├── about.rs               # About window view
    └── create_tunnel.rs       # Create tunnel window view
```

---

## Considerazioni su macOS

### Main Thread Requirement
Su macOS, **tutte le operazioni UI devono avvenire sul main thread**. L'approccio attuale viola questa regola spawning processi separati.

#### iced::daemon Gestisce Questo Automaticamente
- L'event loop di iced gira sempre sul main thread
- `window::open()` viene eseguito sul main thread
- Tutte le view functions sono chiamate dal main thread

### NSApplication Integration
`iced` su macOS crea automaticamente un `NSApplication` e gestisce:
- Main run loop
- Window management
- Event dispatching

---

## Benefici Attesi

### Performance
- ✅ **-66% binari**: Da 3 a 1 eseguibile
- ✅ **-50% build time**: No multiple compilations
- ✅ **Zero polling overhead**: Event-driven
- ✅ **Startup più veloce**: No process spawning

### Manutenibilità
- ✅ **Codice centralizzato**: Tutta la UI in un posto
- ✅ **Type safety**: Shared state compile-time checked
- ✅ **Error handling**: Propagazione errori semplificata
- ✅ **Debugging**: Single process debugging

### Multipiattaforma
- ✅ **macOS compliant**: Main thread rispettato
- ✅ **Windows/Linux**: Funziona out-of-box
- ✅ **Consistent UX**: Stessa logica su tutti gli OS

---

## Rischi e Mitigazioni

### Rischio: Complessità Iniziale
**Mitigazione**: Seguire esempi ufficiali di iced (`examples/multi_window`)

### Rischio: Breaking Changes
**Mitigazione**: Backup del codice, test incrementali

### Rischio: Learning Curve iced
**Mitigazione**: Documentazione eccellente, community attiva

---

## Timeline Stimata

- **Fase 1**: 1 giorno
- **Fase 2**: 3-4 giorni
- **Fase 3**: 1 giorno
- **Fase 4**: 1 giorno
- **Fase 5**: 2-3 giorni

**Totale**: ~8-10 giorni di sviluppo

---

## Conclusioni

La migrazione a `iced::daemon` con multi-window support è la soluzione ottimale per:

1. ✅ Eliminare i binari separati
2. ✅ Rimuovere il polling
3. ✅ Rispettare i requisiti macOS
4. ✅ Migliorare performance e manutenibilità
5. ✅ Mantenere compatibilità multipiattaforma

**Raccomandazione**: Procedere con l'implementazione graduale seguendo le fasi descritte.

---

## Risorse Utili

- [Iced Multi-Window Example](https://github.com/iced-rs/iced/blob/master/examples/multi_window/src/main.rs)
- [Iced Documentation](https://docs.rs/iced)
- [Iced Discourse](https://discourse.iced.rs)
- [tray-icon Docs](https://docs.rs/tray-icon)
- [native-dialog Docs](https://docs.rs/native-dialog)
