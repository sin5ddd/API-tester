# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and the project adheres to Semantic Versioning.

## [v0.1.3] - 2025-10-01
### Fixed
- **Form field value types now properly preserved when sending requests**
  - Previously, when using Form mode with Content-Type: application/json, all field values were converted to strings regardless of their type selector (Float, Int, Bool)
  - Now respects the type selector for each field:
    - Int fields are sent as JSON numbers (integer)
    - Float fields are sent as JSON numbers (floating-point)
    - Bool fields are sent as JSON booleans (true/false)
    - String fields remain as JSON strings
  - Invalid numeric values gracefully fall back to string representation
  - This fix ensures APIs receive correctly typed data instead of everything as strings

## [v0.1.2] - 2025-09-30
### Added
- JSON tree root node now automatically expands when receiving a response for easier immediate access
- Profiles are now stored in user home directory instead of binary directory
    - Windows: `%USERPROFILE%\.api-tester\profiles\`
    - Mac/Linux: `~/.api-tester/profiles/`
- Automatic migration: existing profiles in binary directory are automatically moved to home directory on first launch
- Console window is now hidden on Windows (release builds only; visible in debug builds for debugging)
- Typed form fields with proper JSON type preservation
    - Support for 6 data types: String, Int, Float, Bool, Array, Object
    - Type selector dropdown for each field
    - Numeric values (Int/Float) are no longer escaped with double quotes in JSON
    - Boolean values are properly converted to JSON true/false
    - Array and Object types support nested child fields with recursive structure
    - "+ Add child" button for adding nested fields to Array/Object types
    - Visual indentation for nested structures (20px per level)
    - Automatic type detection when switching from JSON to Form mode
- Advanced JSON editor with syntax highlighting
  - Replaced basic TextEdit with `egui_code_editor` for JSON body editing
  - Syntax highlighting with GRUVBOX color theme
  - Line numbers display
  - 12-row editor with font size 14
  - "Format JSON" button for one-click JSON formatting/prettification
- Content-Type selector in Headers section
  - ComboBox dropdown with 5 common Content-Type options:
    - application/json (default)
    - application/x-www-form-urlencoded
    - text/plain
    - application/xml
    - multipart/form-data
  - Automatically updates Content-Type header in headers text field when changed

### Changed
- Migrated from egui 0.27 to egui 0.32.3
    - Updated eframe, egui, and egui_extras dependencies to version 0.32.3
    - Fixed app creation callback to return `Result` type as required by new eframe API
    - Replaced deprecated `CollapsingHeader::id_source` with `id_salt`
    - Refactored irrefutable pattern matching for improved code clarity
    - All existing functionality preserved; no breaking changes to user features

- Profiles panel now uses auto-shrink behavior
  - Changed from fixed max_height(220.0) to auto_shrink([false, true])
  - Panel automatically expands/contracts based on number of projects and profiles
  - Better screen space utilization
- Headers section now starts collapsed by default
  - Changed from `ui.collapsing()` to `CollapsingHeader::default_open(false)`
  - Reduces visual clutter on startup

### Fixed
- **Form key=value mode now respects Headers' Content-Type setting**
  - Previously, Form mode always sent data as `application/x-www-form-urlencoded` regardless of Headers
  - Now checks Headers for Content-Type and formats body accordingly:
    - If `Content-Type: application/json` is set in Headers, form fields are converted to JSON format
    - Otherwise, uses form-urlencoded format as before
  - Headers information is now properly enforced in all body modes

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
