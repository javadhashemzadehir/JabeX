# JabeX Backend

Offline-first Rust backend hosted inside Tauri 2. Independently testable. No React/Next/UI dependencies.

## Folder map

```
src-tauri/src/
├── main.rs                 # Desktop entry
├── lib.rs                  # Bootstraps DB, tracing, Tauri commands
├── state.rs                # Shared AppState
├── commands/               # Thin IPC adapters
├── domain/                 # Pure models, ports, scheduler, errors
├── application/            # Use-cases / services
├── infrastructure/         # SQLite, filesystem, registry, autostart
├── desktop/                # Tray / window shell (Tauri UI APIs only)
└── shared/                 # IDs (UUIDv7), time helpers
```

## Data flow

1. Frontend invokes a Tauri command
2. Command validates DTO shape and forwards to a service
3. Service applies domain rules (validation, Leitner scheduling)
4. Service persists through repository ports
5. SQLite adapters execute SQL; media files go to `media/`

```
UI ──IPC──> commands ──> application services ──> domain
                                   │
                                   ▼
                         infrastructure (SQLite / FS)
```

## Repository pattern

- Traits live in `domain/repositories.rs` (ports)
- SQLite implementations live in `infrastructure/repositories/`
- Domain never imports `rusqlite`

## Scheduler design

`domain/scheduler.rs` defines:

```rust
pub trait Scheduler {
    fn schedule(&self, card: &Card, grade: Grade, now: DateTime<Utc>) -> ScheduleResult;
}
```

`LeitnerScheduler` is the default. `FsrsScheduler` is a stub reserved for later.

Leitner rules:

- New cards start in box 1
- Correct (`Hard`/`Good`/`Easy`) → box + 1 (max 5)
- Wrong (`Again`) → box 1
- Intervals: 1 / 3 / 7 / 14 / 30 days

## Local data layout

Resolved via `directories` (platform app-data):

```
JabeX/
├── app.sqlite
├── profiles/{uuid}/
│   ├── database.sqlite
│   └── media/
│       ├── images/
│       ├── audio/
│       └── drawings/
└── backups/*.jbx
```

- SQLite: `PRAGMA foreign_keys=ON`, `journal_mode=WAL`
- Media: SHA-256 content-addressed filenames, duplicate-safe
- Backups: versioned `.jbx` (normalized JSON + media), not raw SQLite

## Commands (IPC)

Profiles: `list_profiles`, `get_active_profile`, `create_profile`, `update_profile`, `delete_profile`, `switch_profile`  
Decks: `create_deck`, `list_decks`, `get_deck`, `rename_deck`, `delete_deck`  
Cards: `create_card`, `get_card`, `list_cards`, `update_card`, `delete_card`  
Reviews: `list_due_cards`, `submit_review`  
Settings (per-profile study KV): `get_setting`, `set_setting`  
App preferences (device-level tray/window): `get_app_preferences`, `set_app_preferences`  
Media/Backup: `store_media`, `export_backup`, `validate_backup`, `import_backup` (`create_backup` alias)

Tray menu emits frontend event `tray://action` for UI-only flows (`switch-profile`, settings, restore). Switch Profile only emits the event after showing the window; the frontend opens the existing profile switcher.

## Testing

```bash
cd apps/desktop/src-tauri
cargo test
```

Covers Leitner scheduling, migrations, media dedupe, card CRUD, profile bootstrap, and `.jbx` roundtrip.

## Privacy

- No telemetry / analytics / remote APIs
- No online accounts — local profiles only
- Logs never include card fronts/backs or other study content
