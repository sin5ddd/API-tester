# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and the project adheres to Semantic Versioning.

## [Unreleased]
### Changed
- Migrated from egui 0.27 to egui 0.32.3
  - Updated eframe, egui, and egui_extras dependencies to version 0.32.3
  - Fixed app creation callback to return `Result` type as required by new eframe API
  - Replaced deprecated `CollapsingHeader::id_source` with `id_salt`
  - Refactored irrefutable pattern matching for improved code clarity
  - All existing functionality preserved; no breaking changes to user features

## [v0.1.0] - 2025-09-29
### Added
- Initial release of a native GUI (eframe/egui) REST API tester.
- HTTP methods: GET / POST / PUT / PATCH / DELETE.
- Background request execution to keep the UI responsive.
- Authentication: None / Basic / Bearer / OAuth2 Token.
- Header editor using `Key: Value` format (one per line).
- Body editors:
  - JSON input (pretty-printed).
  - Form key=value editor with compact rows (+Add, delete, enable/disable).
  - Bidirectional sync between JSON and Form (always in sync).
- Response display:
  - Tree view (scrollable for large content).
  - Status colorization (200=green, 3xx/others=yellow, 4xx/5xx=red).
- Diff:
  - Structural diff via `json_patch::diff`.
  - Inline highlighting in the Tree (color per op: add/replace/remove, etc.).
  - For replace ops, show the old value in gray next to the current value.
- Query:
  - JSONPath (extraction).
  - JMESPath (extraction + transformation).
- Search (text): list matching lines within pretty-printed JSON.
- Profiles management (project/folder based):
  - Saved at `./profiles/<project>/<profile>.json`.
  - Folder-like explorer UI (New/Save/Overwrite/Load/Delete/Rename for projects and profiles).
- cURL import (basic parsing for method/url/headers/body).
- Export: JSON, NDJSON (arrays), CSV (flatten arrays of objects).
- Auto polling (configurable interval).
- Window Always-on-Top toggle (Float/Unfloat).
- Compact UI (tight spacing, condensed layout and fonts).

### CI/Release
- Tag push (e.g., `v0.1.0`) triggers GitHub Actions to build for Windows/Linux/macOS.
- Archives (tar.gz) and SHA256 checksums are generated automatically.
- Assets are uploaded to the matching GitHub Release.

### Notes
- On Windows, installing the Visual C++ Redistributable may be necessary.
- Virtualized scrolling for huge arrays is not implemented yet (planned improvement).
