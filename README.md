# Bucket

Personal travel scrapbook desktop app — countries, places, photos, and an inbox for dumped links.

## Prerequisites

- [Node.js](https://nodejs.org/) 18+ (includes npm)
- [Rust](https://www.rust-lang.org/tools/install) stable (rustc + cargo)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually preinstalled on Windows 10/11)

## Run in development

From the project root:

```powershell
npm install
npm run tauri:dev
```

Or double-click **`Start Bucket.bat`**.

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

Use **Library folder** in the app header to open it in Explorer. **Export zip** writes a backup anywhere except inside that folder.

## Features

- Country posters → places → photo gallery
- Notes, been-there, delete
- Inbox: paste YouTube/web URLs (YouTube title/thumb when online) or drop screenshots; file to a country, optionally a place
- Search across countries, places, notes, and unfiled inbox items
- Saved links on countries and places
- Zip export of the whole library
- JPEG, PNG, WebP; HEIC rejected with a clear message

## Project layout

- `src/` — React + TypeScript frontend (Vite)
- `src-tauri/` — Rust backend (Tauri commands, SQLite, image pipeline)
- `Start Bucket.bat` — quick dev launcher for Windows
