# STATE

## Project Reference
Zenith — personal local-first bucket list (Tauri 2 + React + SQLite). Cargo package name is still `bucket`.

## Current Position
Paused 2026-10-07 after cover search paging and cover positioning. No active phase directory.

Progress: cover scroll and drag are installed in Zenith.exe and waiting on the user to try them.

## Recent Decisions
- Library stays `%USERPROFILE%\Documents\Bucket`
- Goals are a sibling of countries, not a kind of country
- Cover photos come from Openverse and are copied into the library
- `cover_x` / `cover_y` are the image point that stays in the middle of the frame
- Header is the word Zenith. The icon is the existing white-ink mountain.

## Pending Todos
- User quits and reopens Zenith, then tries scrolling cover results and dragging a photo.

## Blockers/Concerns
None in code. The Tauri window was not clicked for this pass.

## Session Continuity
Last session: 2026-10-07 — cover paging, cover focus, local install
Stopped at: Waiting for the user to try the installed build
Resume file: `.planning/.continue-here.md`
