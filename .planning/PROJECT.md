# Bucket

## What This Is

A personal Windows desktop scrapbook for travel dreams. Country posters → flat place grid → place pages with photos, notes, and saved links, plus an inbox for dumped URLs. Local-only; double-click to open.

## Core Value

See and file the places you want to go — visually, on this computer — without a trip planner or a browser URL.

## Requirements

### Validated / Active (v1)
- Country wall, country magazine (flat places), place gallery
- Photo import: drag-drop, clipboard, file picker; JPEG/PNG/WebP; HEIC rejected
- Library in `%USERPROFILE%\Documents\Bucket`
- Inbox + YouTube oEmbed + triage (place optional) — not built yet
- Search, zip export, editorial visual pass — not built yet

### Out of Scope (v1)
- Area chapters, activities entity, maps
- Trip planner, budget, live flight/hotel prices
- Ferriss dreamlining, cloud sync, accounts, phone

## Key Decisions
- Scrapbook first, not a travel agency
- rusqlite + versioned migrations; relative image paths; asset protocol
- Workhorse for grunt; Claude for later visual polish

## Constraints
- Do not re-scaffold the Tauri app
- Do not add areas / activities / maps / sync in v1
