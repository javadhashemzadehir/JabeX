JabeX — portable Windows build
================================

Private, offline-first digital Leitner Box. No installer, no admin rights.

System requirements
-------------------
- Windows 10 or Windows 11, 64-bit (x64)
- Microsoft Edge WebView2 Runtime (see "WebView2 requirements.txt")

How to run
----------
1. Ensure WebView2 Runtime is installed.
2. Double-click JabeX.exe (or run it from a terminal). No administrator rights required.
3. Create a local profile and start studying.

Where your data is stored
-------------------------
JabeX does NOT write decks, cards, or SQLite files into this folder.

Study data lives in your user application data directory, for example:

  %APPDATA%\JabeX\JabeX\data\

That folder contains:
  - app.sqlite          (local profiles registry)
  - profiles\<id>\      (per-profile SQLite + media)
  - backups\            (optional .jbx exports)

You can move or delete this portable folder without deleting your study data.
To wipe study data, remove the AppData path above (or use Trash / delete profile in-app).

Optional: override data location
--------------------------------
Set environment variable JABEX_DATA_DIR to an absolute folder before launching
if you need an isolated data root (useful for testing). Still never writes into
this portable directory unless you point JABEX_DATA_DIR at it on purpose.

Backup / import
---------------
Use Settings or Home → Backup to export/import .jbx files. Import creates a new
local profile by default. No cloud account is involved.

Folder layout
-------------
  JabeX.exe
  resources\            (reserved; UI assets are embedded in the executable)
  WebView2 requirements.txt
  README.txt            (this file)

Version
-------
See the parent folder name (JabeX-<version>-portable).
