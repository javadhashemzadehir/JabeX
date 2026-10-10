# ADR 011: Smart Language Assist (Optional, Local-First)

## Status

Accepted

## Context

Language learners often write foreign vocabulary on cards, including Persian-script transliterations of English (and other) words. Helping with detection, translation, and pronunciation improves study quality — but JabeX is privacy-first and offline-first ([ADR 003](../003-local-first-storage/003-local-first-storage.md)). Network must never be mandatory, and failures must never block card creation.

## Decision

Add an optional **Smart Language Assist** module behind Clean Architecture ports ([ADR 007](../007-clean-architecture/007-clean-architecture.md)):

| Port | Role |
|------|------|
| `LanguageDetectionService` | Offline script/heuristic detection + Persian transliteration map |
| `TranslationProvider` | Optional translation (`none` or `google`) |
| `SpeechProvider` | Optional TTS (`none` or `google`) |
| `PronunciationAudioStorage` | Cache audio under `media/audio/pronunciation/` |

### Behaviour

1. **Always local:** On card create/update, a background job runs heuristic language detection and transliteration lookup. No network required.
2. **Opt-in network (profile master):** Profile settings (`settings.language_assist`) control:
   - `autoTranslation` (default `false`) — text translation master
   - `pronunciationGeneration` (default `false`) — spoken TTS master
   - `provider` (`none` | `google`, default `none`)
   - `targetLanguage` (default `fa`)
3. **Per-deck / per-card overrides:** After the profile master is on, each deck (`decks.settings.languageAssist`) and card (`language_assist_overrides`) can set `translation` / `pronunciation` to `inherit` | `enabled` | `disabled`. Resolution is card → deck → profile. Example: deck pronunciation enabled, 8 cards `disabled`, 2 cards `inherit` → only two cards get spoken audio.
4. **Non-blocking:** Card CRUD returns immediately. Assist runs via `tokio` / `spawn_blocking`. Network errors set `language_assist_status = failed` and are logged; the card remains usable.
5. **Storage:** Card columns hold `detected_language`, `translation`, `pronunciation_audio_path`, `transliteration_json`, `language_assist_status`, `language_assist_overrides_json`. Audio bytes live on disk, not in SQLite ([ADR 010](../010-rich-content-storage/010-rich-content-storage.md) pattern).

### Google adapters

Infrastructure adapters call public Google Translate / TTS HTTP endpoints **without API keys or accounts**. They are best-effort, may change, and are only invoked when the user enables a feature and selects `google`. No analytics SDK is introduced.

### On-disk audio layout

```
profiles/{profile_id}/media/audio/pronunciation/{card_id}.mp3
```

## Consequences

### Positive

- Vocabulary cards get language metadata offline
- Translation / audio remain optional and user-controlled
- Clean Architecture keeps providers swappable
- Study path stays free of mandatory networking

### Negative

- Unofficial Google endpoints can break without notice
- Background jobs may briefly show `pending` status
- Audio cache grows with usage (user can disable generation)

## Alternatives Considered

- **Mandatory cloud translation API with keys:** Rejected (accounts, secrets, privacy)
- **Embed large offline NMT/TTS models:** Deferred (binary size, licensing)
- **Store audio as SQLite BLOBs:** Rejected (same rationale as ADR 010)
