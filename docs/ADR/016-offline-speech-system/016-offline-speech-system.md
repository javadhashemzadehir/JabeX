# ADR 016: Offline Speech & Language Learning System

## Status

Accepted

## Context

[ADR 011](../011-smart-language-assist/011-smart-language-assist.md) added optional language assist with local detection and opt-in unofficial Google Translate/TTS. Learners need **fully offline** translation and pronunciation that:

- Never require API keys or accounts
- Never send card text to the network by default
- Work on Windows with zero install (SAPI)
- Allow higher-quality local TTS (Piper) when the user downloads models
- Cache audio and translations on disk / SQLite

Embedding large NMT models in the installer was previously deferred (binary size). This ADR adopts a **hybrid offline stack** that keeps the installer small while remaining production-ready.

## Decision

Evolve Smart Language Assist into an offline-first speech pipeline behind existing Clean Architecture ports ([ADR 007](../007-clean-architecture/007-clean-architecture.md)):

```
Language Detection → Translation Engine → Pronunciation (IPA)
                              ↓
                     TTS Provider chain
                              ↓
                     Audio Cache Manager
                              ↓
                     Flashcard / Review UI
```

### TTS providers (`TtsProvider`)

| Provider | Role |
|----------|------|
| `PiperProvider` | Optional local Piper binary + ONNX voices under `{app_data}/JabeX/data/voices/{lang}/` |
| `WindowsSapiProvider` | Windows System.Speech (no install) |
| Browser Speech API | Frontend fallback when no native WAV is available |

**Fallback order (`auto`):** Piper → Windows SAPI → Browser.

### Translation providers

| Provider | Role |
|----------|------|
| Offline dictionary + `translation_cache` | Default; zero network |
| Optional `argos-translate` CLI | Used only if installed on PATH |
| Legacy Google | Kept in code for stored settings; not exposed as the default UI path |

### Persistence (migration V11)

- `card_audio` — content-addressed audio metadata (`hash` = SHA-256 of text+language+voice+speed)
- `language_preferences` — per-language voice/speed/provider
- `translation_cache` — offline translation results
- `cards.pronunciation_ipa` — IPA hint when known

Audio bytes remain under the profile media tree ([ADR 010](../010-rich-content-storage/010-rich-content-storage.md)):

```
media/audio/pronunciation/{card_id}.wav
media/audio/cache/{sha256}.wav
```

Voice models (optional, user-downloaded):

```
{app_data}/JabeX/data/
├── piper/          # piper.exe (optional)
└── voices/
    ├── en/
    ├── fa/
    ├── fr/
    └── …
```

### Layout note

Rust code lives in `domain/speech`, `application/language_assist_service`, and `infrastructure/providers/*` — **not** a parallel `src-tauri/src/features/speech` tree — to preserve Clean Architecture. Frontend UI modules live under `frontend/src/features/speech/` plus the existing `language-assist` panels.

### Behaviour

1. Card create/update still returns immediately; assist runs on a blocking worker.
2. Detection + IPA always run locally.
3. Translation / TTS run only when profile (+ deck/card) toggles allow.
4. Identical utterances reuse `card_audio` / disk cache (never regenerate).
5. Piper generation can be cancelled; queue is single-flight per provider instance.

## Consequences

### Positive

- Core speech features work offline with no keys
- Installer stays small (Piper models optional)
- Providers remain swappable behind ports
- Extends ADR 011 without breaking card CRUD

### Negative

- Bundled dictionary coverage is intentionally limited; Argos/Piper require user installs
- SAPI quality and language coverage depend on Windows voices
- Legacy Google settings may still exist on upgraded profiles

## Alternatives Considered

- **Ship full Argos + Piper inside the MSI:** Rejected (hundreds of MB; licensing/update burden)
- **Mandatory cloud TTS/NMT:** Rejected (privacy, keys, offline-first)
- **Parallel `features/speech` Rust crate tree:** Rejected (violates ADR 007 dependency rules)
