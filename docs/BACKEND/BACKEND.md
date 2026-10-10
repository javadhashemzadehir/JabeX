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
│       ├── attachments/{uuid}/…
│       ├── ink/{uuid}.json
│       └── audio/pronunciation/{card}.mp3
└── backups/*.jbx
```

- SQLite: `PRAGMA foreign_keys=ON`, `journal_mode=WAL`
- Media: UUID folders + SHA-256 checksums (metadata in SQLite; binaries on disk)
- Ink: vector stroke JSON files; SQLite holds references + content hash
- Backups: versioned `.jbx` (normalized JSON + media + checksums), not raw SQLite
- Backup UI uses native open/save dialogs (`tauri-plugin-dialog`); paths are passed into `export_backup` / `import_backup`

## Commands (IPC)

Profiles: `list_profiles`, `get_active_profile`, `create_profile`, `update_profile`, `delete_profile`, `switch_profile`  
Decks: `create_deck`, `list_decks`, `get_deck`, `rename_deck`, `delete_deck`  
Cards: `create_card`, `get_card`, `list_cards`, `update_card`, `delete_card` (faces are versioned TipTap JSON `document_v2`; see ADR 009 / ADR 014)  
Language assist: `get_language_assist_settings`, `set_language_assist_settings`, `regenerate_language_assist`, `read_pronunciation_audio` (ADR 011; create/update enqueue background detection)  
Reviews: `list_due_cards`, `submit_review` (also projects into `study_activity` via `ReviewCompletedEvent`)  
Stats: `get_study_stats` (local aggregates: decks, cards, due, reviews today, box distribution)  
Study consistency: `get_study_consistency_stats`, `list_study_activity` (streaks / heatmap; ADR 013)  
App preferences (device-level tray/window): `get_app_preferences`, `set_app_preferences`  
Backup: `export_backup`, `validate_backup`, `import_backup` (tray Create Backup calls `export_backup` in-process)  
Attachments: `create_attachment`, `list_attachments`, `delete_attachment`, `restore_attachment`, `read_attachment_bytes`  
Media library: `list_media_library`, `get_media_library_stats`, `purge_attachment`, `purge_deleted_attachments`, `remove_orphan_media_dirs`, `verify_media_library`  
Ink: `create_ink_document`, `get_ink_document`, `list_ink_documents`, `save_ink_document`, `delete_ink_document`  
Trash: `list_deleted_items`, `restore_item`, `permanently_delete_item` (ADR 012; `delete_deck` / `delete_card` soft-delete with cascade)  
Search: `search_global`, `rebuild_search_index`, `list_recent_searches`, `clear_recent_searches`, `delete_recent_search`, `list_popular_searches`, `record_search_selection`, `list_search_history`, `clear_search_history` (ADR 015; FTS5 index, local-only)

Tray menu emits frontend event `tray://action` for UI-only flows (`switch-profile`, settings, restore). Switch Profile only emits the event after showing the window; the frontend opens the existing profile switcher.

Language assist emits `language-assist://updated` when a background job finishes so the UI can refresh card metadata.

## Testing

```bash
cd apps/desktop/src-tauri
cargo test
```

Covers Leitner scheduling, migrations, media dedupe, card CRUD, profile bootstrap, Trash restore/cascade/permanent delete, and `.jbx` roundtrip (including soft-deleted entities).

## Privacy

- No telemetry / analytics SDKs
- No online accounts — local profiles only
- Core study path has no mandatory network calls
- Optional Smart Language Assist may call Google Translate/TTS **only** when the user enables it in settings (ADR 011)
- Logs never include card fronts/backs or other study content
