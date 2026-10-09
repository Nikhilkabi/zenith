# Bucket

Personal travel scrapbook desktop app — countries, places, photos, and an inbox for dumped links.

## Prerequisites

- [Node.js](https://nodejs.org/) 18+ (includes npm)
- [Rust](https://www.rust-lang.org/tools/install) stable (rustc + cargo)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually preinstalled on Windows 10/11)

## Run the app

Double-click **Bucket** on your Desktop. That opens the compiled app (no terminal, no wait for Cargo).

The installed copy lives at:

`%LOCALAPPDATA%\Programs\Bucket\Bucket.exe`

After you change the code, rebuild and install:

```powershell
npm run install:local
```

## Develop (optional)

```powershell
npm install
npm run tauri:dev
```

## Build a desktop app

```powershell
npm run tauri:build
```

The installer/exe output lands in `src-tauri/target/release/bundle/`.

## Where your data lives

```
%USERPROFILE%\Documents\Bucket\
├── db.sqlite
└── library\
```

Use **Library folder** in the app header to open it in Explorer. **Export zip** writes a backup anywhere except inside that folder. **Import zip** merges files into the same library.

## Features

- Country posters → places → photo gallery
- Notes, been-there, dream/been filters, drag-reorder
- Inbox: paste any URL (YouTube or Open Graph title/thumb when online) or drop screenshots; file later
- Search across countries, places, notes, links, and unfiled inbox items
- Saved links on countries and places
- Photo captions, EXIF date, country photo strip
- Trash with restore (purge after 30 days)
- Zip export / import of the library
- JPEG, PNG, WebP, AVIF (converted); HEIC rejected with a clear message

## Keys

Press `?` in the app for the cheat sheet. `/` focuses search.

## Project layout

- `src/` — React + TypeScript frontend (Vite)
- `src-tauri/` — Rust backend (Tauri commands, SQLite, image pipeline)
- `Start Bucket.bat` — quick dev launcher for Windows
