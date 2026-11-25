# Changelog

All notable changes to this project are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---
## [0.5.0] — 2026

### Changed
- Migrated to `hazard3-st7789v-graphics` and the `hazard3_st7789v_graphics` import in the independent Hazard3 Async ecosystem.
- Replaced the `embassy-st7789v` driver with `hazard3-st7789v` 0.8.
- The default backend is `time-hazard3`; `time-upstream` enables upstream Embassy when default features are disabled.
- Re-exported driver types so examples and applications can import them from the graphics crate.
- The old `embassy-st7789v-graphics` releases were yanked from new dependency resolution.

## [0.4.0] — 2026

### Added
- Re-exported mathematical and geometric constants from the graphics crate:
  - `LAMBDA`, `THETA`, `PI`, `DELTA`, `DEGREE`, `PLUS_MINUS`, `TIMES`, `DIVIDE`, `SQRT`, `INFINITY`, `APPROX`, `LE`, `GE`
- Updated documentation and examples to import these symbols directly from `embassy_st7789v_graphics`.

### Changed
- Updated the `embassy-st7789v` dependency to version 0.5.

## [0.3.0] — 2026

### Changed
- Updated the `embassy-st7789v` dependency to version 0.4.0.
- No API changes for this crate.

## [0.2.0] — 2026

### Added
- `GraphicsBuffered` — graphics context for `St7789vBuffered` (RAM framebuffer)
  - `new` and `new_no_rst`
  - `pixel` — synchronous RAM write with clipping
  - `hline` — horizontal RAM line via `fill_rect_buf`
- Synchronous framebuffer primitives (zero flicker, no `.await`):
  - `line_buf`
  - `circle_buf`
  - `fill_circle_buf`
  - `triangle_buf`
  - `fill_triangle_buf`
  - `ellipse_buf`
  - `bezier_quad_buf`

### Changed
- No changes to the existing API — fully backward compatible.

---

## [0.1.0] — 2026

### Added
- `Graphics<'a, SPI, DC, RST>` — graphics context with automatic clipping
- `Graphics::new` and `Graphics::new_no_rst`
- Async primitives (direct SPI):
  - `line` — Bresenham integer-only
  - `circle` — midpoint circle integer-only (8-octant symmetry)
  - `fill_circle` — midpoint + hlines via `fill_rect`
  - `triangle` — outline (3 calls to `line`)
  - `fill_triangle` — scanline integer-only
  - `ellipse` — midpoint ellipse integer-only (2 regions)
  - `bezier_quad` — De Casteljau integer-only fixed-point ×1024