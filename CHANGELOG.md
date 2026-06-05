# Changelog

Toutes les modifications notables de ce projet sont documentées ici.
Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

---

## [0.2.0] — 2026

### Ajouté
- `GraphicsBuffered` — contexte graphique pour `St7789vBuffered` (framebuffer RAM)
  - `new` et `new_no_rst`
  - `pixel` — écriture RAM avec clipping, synchrone
  - `hline` — ligne horizontale RAM via `fill_rect_buf`
- Primitives framebuffer synchrones (zéro clignotement, zéro `.await`) :
  - `line_buf`
  - `circle_buf`
  - `fill_circle_buf`
  - `triangle_buf`
  - `fill_triangle_buf`
  - `ellipse_buf`
  - `bezier_quad_buf`

### Modifié
- Aucun changement d'API existante — compatibilité ascendante totale

---

## [0.1.0] — 2026

### Ajouté
- `Graphics<'a, SPI, DC, RST>` — contexte graphique avec clipping automatique
- `Graphics::new` et `Graphics::new_no_rst`
- Primitives async (direct SPI) :
  - `line` — Bresenham integer-only
  - `circle` — midpoint circle integer-only (symétrie 8-octants)
  - `fill_circle` — midpoint + hlines via `fill_rect`
  - `triangle` — contour (3 appels à `line`)
  - `fill_triangle` — scanline integer-only
  - `ellipse` — midpoint ellipse integer-only (2 régions)
  - `bezier_quad` — De Casteljau integer-only virgule fixe ×1024