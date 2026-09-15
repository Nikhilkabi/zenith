# Bucket

Personal travel scrapbook desktop app — collect countries, places, and photos in a local library folder.

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

Or double-click **`Start Bucket.bat`** — it launches `npm run tauri:dev` in a minimized console window.

## Build a desktop app

```powershell
npm run tauri:build
```

The installer/exe output lands in `src-tauri/target/release/bundle/`.

## Where your data lives

Bucket stores everything under:

```
%USERPROFILE%\Documents\Bucket\
├── db.sqlite
└── library\
```

- **Database:** `db.sqlite` (SQLite with versioned migrations)
- **Photos:** copied into `library/images/{place_id}/` with original, display (~2000px), and thumb (~600px) variants
- Relative paths are stored in the database; the UI loads images through Tauri's asset protocol

Use **Open library folder** in the app header to reveal the folder in Explorer.

## Current features (v1 slice)

- Create and browse countries
- Add places per country (dream / been)
- Import photos via drag-drop, clipboard paste (Ctrl+V), or file picker
- JPEG, PNG, WebP supported; HEIC rejected with a clear message

## Project layout

- `src/` — React + TypeScript frontend (Vite)
- `src-tauri/` — Rust backend (Tauri commands, SQLite, image pipeline)
- `Start Bucket.bat` — quick dev launcher for Windows
