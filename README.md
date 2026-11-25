# hazard3-st7789v-graphics

A `no_std` 2D graphics layer for **ST7789V 240×320** TFT LCD displays,
built on [`hazard3-st7789v`](https://crates.io/crates/hazard3-st7789v).
It is part of [Hazard3 Async](https://github.com/jorgeandrecastro/hazard3-async),
an independent ecosystem for the RP2350's RISC-V Hazard3 cores,
not affiliated with the Embassy project.

## Features

The crate re-exports the driver types and the character constants
`LAMBDA`, `THETA`, `PI`, `DELTA`, `DEGREE`, `PLUS_MINUS`, `TIMES`, `DIVIDE`,
`SQRT`, `INFINITY`, `APPROX`, `LE` and `GE`. See the [changelog](CHANGELOG.md).

| Function               | Algorithm                   | Direct SPI | Framebuffer |
|------------------------|-----------------------------|:----------:|:-----------:|
| `line`                 | Bresenham integer-only      | ✓ async    | ✓ sync      |
| `circle`               | Midpoint integer-only       | ✓ async    | ✓ sync      |
| `fill_circle`          | Midpoint + hlines SPI batch | ✓ async    | ✓ sync      |
| `triangle`             | 3 × `line`                  | ✓ async    | ✓ sync      |
| `fill_triangle`        | Scanline integer-only       | ✓ async    | ✓ sync      |
| `ellipse`              | Generalized midpoint        | ✓ async    | ✓ sync      |
| `bezier_quad`          | De Casteljau integer-only   | ✓ async    | ✓ sync      |

All primitives:

- Zero allocation (`no_std`, no `Vec`)
- No floating-point operations (safe on MCUs without an FPU)
- Automatic clipping to 240×320
- RGB565 `Color` values (the same type as the driver)

---

## Two usage modes

### Direct SPI mode (`Graphics`)

Each primitive sends pixels immediately over SPI.
This is simple, but may cause flicker if the display is redrawn frequently.

### Framebuffer mode (`GraphicsBuffered`)

All primitives write to RAM (synchronously, without `.await`).
A `flush()` call sends the frame, reducing visible incremental drawing
without guaranteeing tear-free updates. `St7789vBuffered` requires 150 KiB
of RAM; ensure sufficient memory and stack space for construction.

---

## Quick start

```toml
[dependencies]
hazard3-st7789v-graphics = "0.5"
```

The default `time-hazard3` backend uses `hazard3-time`. The application must
provide a compatible time driver, executor and configuration:
see the [Hazard3 Async quick start](https://github.com/jorgeandrecastro/hazard3-async#quick-start).
If the application also uses the driver directly, select
`hazard3-st7789v = "0.8"` and the same backend for both dependencies.

### Alternative: upstream Embassy

For an application using the official Embassy time backend, disable default
features and select `time-upstream`:

```toml
[dependencies]
hazard3-st7789v-graphics = { version = "0.5", default-features = false, features = ["time-upstream"] }
# If the driver is also a direct dependency:
hazard3-st7789v = { version = "0.8", default-features = false, features = ["time-upstream"] }
```

Exactly one backend must be enabled, with a matching time driver and runtime.
Do not mix Hazard3 and upstream Embassy time drivers; Cargo features are
additive across dependencies.

### Migration from `embassy-st7789v-graphics`

The old releases have been yanked from new dependency resolution, not deleted:
existing `Cargo.lock` files can still use them. Replace the package and import
with `hazard3-st7789v-graphics` and `hazard3_st7789v_graphics` instead of
`embassy-st7789v-graphics` and `embassy_st7789v_graphics`, and replace
`embassy-st7789v` with `hazard3-st7789v`. Driver types must come from the new
crate. Hazard3 version 0.5.0 must be published before installation from
crates.io works; see also the [migration table](https://github.com/jorgeandrecastro/hazard3-async).

### Direct mode (async)

```rust,ignore
use hazard3_st7789v_graphics::{
    circle, fill_triangle, line, Color, Graphics, St7789v, PI, DEGREE,
};

let mut display = St7789v::new(spi_device, dc_pin, rst_pin);
display.init().await.unwrap();
display.fill_screen(Color::BLACK).await.unwrap();

{
    let mut gfx = Graphics::new(&mut display);
    line(&mut gfx, 0, 0, 239, 319, Color::WHITE).await;
    circle(&mut gfx, 120, 160, 60, Color::CYAN).await;
    fill_triangle(&mut gfx, 120, 10, 20, 310, 220, 310, Color::YELLOW).await;
} // ← borrow released

display.draw_str(8, 8, b"HELLO", Color::WHITE, Color::BLACK).await.unwrap();
```

### Framebuffer mode (sync + flush)

```rust,ignore
use hazard3_st7789v_graphics::{
    circle_buf, fill_triangle_buf, line_buf, Color, GraphicsBuffered, St7789v,
    St7789vBuffered, PI, DEGREE,
};

let mut display = St7789vBuffered::new(St7789v::new(spi_device, dc_pin, rst_pin));
display.driver().init().await.unwrap();

loop {
    display.fill_screen_buf(Color::BLACK);

    {
        let mut gfx = GraphicsBuffered::new(&mut display);
        line_buf(&mut gfx, 0, 0, 239, 319, Color::WHITE);
        circle_buf(&mut gfx, 120, 160, 60, Color::CYAN);
        fill_triangle_buf(&mut gfx, 120, 10, 20, 310, 220, 310, Color::YELLOW);
    } // ← borrow released

    display.draw_str_buf(8, 8, b"HELLO", Color::WHITE, Color::BLACK);
    display.flush().await.unwrap();
}
```

---

## Without an RST pin

```rust,ignore
let mut display = St7789v::new_no_rst(spi_device, dc_pin);
let mut gfx = Graphics::new_no_rst(&mut display);

// Framebuffer without RST
let mut display = St7789vBuffered::new(St7789v::new_no_rst(spi_device, dc_pin));
let mut gfx = GraphicsBuffered::new_no_rst(&mut display);
```

---

## SPI errors

Direct graphics primitives ignore SPI errors. If the application needs to
handle them, use the driver methods that return a `Result` directly. The
`*_buf` primitives only write to memory; `flush()` returns SPI errors.

## License

GPL-2.0-or-later — Copyright (C) 2026 Jorge Andre Castro