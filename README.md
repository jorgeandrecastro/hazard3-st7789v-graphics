# embassy-st7789v-graphics

Couche graphique 2D `no_std` pour écrans TFT LCD **ST7789V 240×320**,
construite au-dessus de [`embassy-st7789v`](https://crates.io/crates/embassy-st7789v).

## Fonctionnalités nouvelles : Version 0.4.0 (voir CHANGELOG.md)

Cette version re-exporte directement depuis `embassy-st7789v-graphics` les constantes mathématiques et géométriques `LAMBDA`, `THETA`, `PI`, `DELTA`, `DEGREE`, `PLUS_MINUS`, `TIMES`, `DIVIDE`, `SQRT`, `INFINITY`, `APPROX`, `LE` et `GE`, et met à jour la dépendance `embassy-st7789v` vers la version 0.5.

| Fonction               | Algorithme                  | Direct SPI | Framebuffer |
|------------------------|-----------------------------|:----------:|:-----------:|
| `line`                 | Bresenham integer-only      | ✓ async    | ✓ sync      |
| `circle`               | Midpoint integer-only       | ✓ async    | ✓ sync      |
| `fill_circle`          | Midpoint + hlines SPI batch | ✓ async    | ✓ sync      |
| `triangle`             | 3 × `line`                  | ✓ async    | ✓ sync      |
| `fill_triangle`        | Scanline integer-only       | ✓ async    | ✓ sync      |
| `ellipse`              | Midpoint généralisé         | ✓ async    | ✓ sync      |
| `bezier_quad`          | De Casteljau integer-only   | ✓ async    | ✓ sync      |

Toutes les primitives :

- Zéro allocation (`no_std`, pas de `Vec`)
- Zéro flottant (sûr sur MCU sans FPU)
- Clipping automatique sur 240×320
- Couleurs `Color` RGB565 (même type que le driver)

---

## Deux modes d'utilisation

### Mode direct SPI (`Graphics`)

Chaque primitive envoie les pixels immédiatement via SPI.
Simple, mais peut produire un clignotement si l'écran est redessiné fréquemment.

### Mode framebuffer (`GraphicsBuffered`)

Toutes les primitives écrivent en RAM (synchrones, sans `.await`).
Un seul appel `flush()` envoie tout d'un coup  **zéro clignotement**.
Nécessite `St7789vBuffered` du crate `embassy-st7789v` (150 Ko RAM).

---

## Démarrage rapide

```toml
[dependencies]
embassy-st7789v          = "0.5"
embassy-st7789v-graphics = "0.4"
```

### Mode direct (async)

Les constantes mathématiques et géométriques `LAMBDA`, `THETA`, `PI`, `DELTA`, `DEGREE`, `PLUS_MINUS`, `TIMES`, `DIVIDE`, `SQRT`, `INFINITY`, `APPROX`, `LE` et `GE` sont désormais re-exportées par `embassy-st7789v-graphics`, donc vous pouvez les importer directement depuis cette crate.

```rust
use embassy_st7789v_graphics::{
    circle, fill_triangle, line, Color, Graphics, St7789v, PI, DEGREE,
};

let mut ecran = St7789v::new(spi_device, broche_dc, broche_rst);
ecran.init().await.unwrap();
ecran.fill_screen(Color::BLACK).await.unwrap();

{
    let mut gfx = Graphics::new(&mut ecran);
    line(&mut gfx, 0, 0, 239, 319, Color::WHITE).await;
    circle(&mut gfx, 120, 160, 60, Color::CYAN).await;
    fill_triangle(&mut gfx, 120, 10, 20, 310, 220, 310, Color::YELLOW).await;
} // ← borrow libéré

ecran.draw_str(8, 8, b"BONJOUR", Color::WHITE, Color::BLACK).await.unwrap();
```

### Mode framebuffer (sync + flush) : zéro clignotement

```rust
use embassy_st7789v_graphics::{
    circle_buf, fill_triangle_buf, line_buf, Color, GraphicsBuffered, St7789v,
    St7789vBuffered, PI, DEGREE,
};

let mut ecran = St7789vBuffered::new(St7789v::new(spi_device, broche_dc, broche_rst));
ecran.driver().init().await.unwrap();

loop {
    ecran.fill_screen_buf(Color::BLACK);

    {
        let mut gfx = GraphicsBuffered::new(&mut ecran);
        line_buf(&mut gfx, 0, 0, 239, 319, Color::WHITE);
        circle_buf(&mut gfx, 120, 160, 60, Color::CYAN);
        fill_triangle_buf(&mut gfx, 120, 10, 20, 310, 220, 310, Color::YELLOW);
    } // ← borrow libéré

    ecran.draw_str_buf(8, 8, b"BONJOUR", Color::WHITE, Color::BLACK);
    ecran.flush().await.unwrap(); // envoi unique → pas de clignotement
}
```

---

## Sans broche RST

```rust
let mut ecran = St7789v::new_no_rst(spi_device, broche_dc);
let mut gfx = Graphics::new_no_rst(&mut ecran);

// Framebuffer sans RST
let mut ecran = St7789vBuffered::new(St7789v::new_no_rst(spi_device, broche_dc));
let mut gfx = GraphicsBuffered::new_no_rst(&mut ecran);
```

---

## Licence

GPL-2.0-or-later — Copyright (C) 2026 Jorge Andre Castro