#![no_std]
#![forbid(unsafe_code)]
//! # hazard3-st7789v-graphics
//!
//! A `no_std` 2D graphics layer for ST7789V 240×320 TFT LCD displays,
//! built on [`hazard3-st7789v`](https://docs.rs/hazard3-st7789v)
//! in the independent [Hazard3 Async](https://github.com/jorgeandrecastro/hazard3-async) ecosystem.
//! The `time-hazard3` time backend is enabled by default; to use upstream
//! Embassy, disable default features and enable `time-upstream`.
//! Exactly one backend must be enabled.
//!
//! ## Role of this crate
//!
//! The `hazard3-st7789v` driver already provides:
//! - `draw_pixel()`, `draw_hline()`, `draw_vline()`
//! - `draw_rect()`, `fill_rect()`, `fill_screen()`
//! - `draw_char()`, `draw_str()`, `draw_i16()`, `draw_u32()`, `draw_f32()`
//! - `draw_char_scaled()`, `draw_str_scaled()`, `draw_str_scaled_buf()`, `draw_char_scaled_buf()`
//! - `draw_bitmap()`
//! - `set_orientation()`, `set_invert()`
//!
//! This crate **does not duplicate these**. It only adds geometric
//! primitives that the driver does not provide:
//!
//! | Function             | Algorithm                     | Direct SPI | Framebuffer |
//! |----------------------|-------------------------------|:----------:|:-----------:|
//! | [`line()`]           | Bresenham integer-only        | ✓ async    | ✓ sync      |
//! | [`circle()`]         | Midpoint integer-only         | ✓ async    | ✓ sync      |
//! | [`fill_circle()`]    | Midpoint + hlines             | ✓ async    | ✓ sync      |
//! | [`triangle()`]       | 3 calls to [`line()`]         | ✓ async    | ✓ sync      |
//! | [`fill_triangle()`]  | Scanline integer-only         | ✓ async    | ✓ sync      |
//! | [`ellipse()`]        | Generalized midpoint          | ✓ async    | ✓ sync      |
//! | [`bezier_quad()`]    | De Casteljau integer-only     | ✓ async    | ✓ sync      |
//!
//! ## Two usage modes
//!
//! ### Direct SPI mode — [`Graphics`]
//!
//! Each primitive sends pixels immediately over SPI.
//! This is simple, but may cause flicker if the display is redrawn frequently.
//!
//! ### Framebuffer mode — [`GraphicsBuffered`]
//!
//! All primitives write to RAM (synchronously, without `.await`).
//! A [`St7789vBuffered::flush`] call sends the frame, reducing visible
//! incremental drawing without guaranteeing tear-free updates.
//! Requires [`St7789vBuffered`] from `hazard3-st7789v` (150 KiB RAM).
//!
//! The mathematical and geometric constants
//! [`LAMBDA`], [`THETA`], [`PI`], [`DELTA`], [`DEGREE`], [`PLUS_MINUS`],
//! [`TIMES`], [`DIVIDE`], [`SQRT`], [`INFINITY`], [`APPROX`], [`LE`] and
//! [`GE`] are re-exported by this crate, along with basic types such as
//! [`Color`], [`St7789v`] and [`St7789vBuffered`]. You can therefore import
//! them directly from [`hazard3_st7789v_graphics`](crate) without importing
//! `hazard3-st7789v`.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      Your application                           │
//! │  line() / circle() / triangle() …        (async, direct SPI)   │
//! │  line_buf() / circle_buf() / … + flush() (sync,  framebuffer)  │
//! │  display.draw_str() / display.draw_f32() …   (driver direct)       │
//! └──────────┬──────────────────────────┬────────────────────────────┘
//!            │ &mut Graphics             │ &mut GraphicsBuffered
//! ┌──────────▼──────────────┐  ┌────────▼────────────────────┐
//! │  Graphics               │  │  GraphicsBuffered           │
//! │  clipping · pixel async │  │  clipping · pixel sync RAM  │
//! └──────────┬──────────────┘  └────────┬────────────────────┘
//!            │ draw_pixel() async        │ set_pixel() / fill_rect_buf()
//! ┌──────────▼──────────────┐  ┌────────▼────────────────────┐
//! │  St7789v (driver)       │  │  St7789vBuffered            │
//! │  SPI · RAMWR direct     │  │  RAM [u8; 153600] + flush() │
//! └─────────────────────────┘  └─────────────────────────────┘
//! ```
//!
//! ## Borrowing pattern: direct mode
//!
//! `Graphics` holds a `&mut St7789v` for its entire lifetime.
//! To call driver methods directly (text, fill, orientation, etc.),
//! `gfx` must first go out of scope.
//!
//! ```rust,no_run
//! # use embedded_hal::digital::OutputPin;
//! # use embedded_hal_async::spi::SpiDevice;
//! # use hazard3_st7789v_graphics::{circle, line, Color, Graphics, St7789v};
//! # async fn demo<SPI: SpiDevice, DC: OutputPin, RST: OutputPin>(mut display: St7789v<SPI, DC, RST>) {
//! loop {
//!     display.fill_screen(Color::BLACK).await.unwrap();
//!     {
//!         let mut gfx = Graphics::new(&mut display);
//!         line(&mut gfx, 0, 0, 239, 319, Color::WHITE).await;
//!         circle(&mut gfx, 120, 160, 60, Color::CYAN).await;
//!     } // ← borrow released
//!     display.draw_str(8, 10, b"HELLO", Color::YELLOW, Color::BLACK).await.unwrap();
//! }
//! # }
//! ```
//!
//! ## Borrowing pattern: framebuffer mode
//!
//! `GraphicsBuffered` holds a `&mut St7789vBuffered`.
//! All `*_buf` primitives are synchronous.
//! Call `flush()` after releasing `gfx`.
//!
//! ```rust,no_run
//! # use embedded_hal::digital::OutputPin;
//! # use embedded_hal_async::spi::SpiDevice;
//! # use hazard3_st7789v_graphics::{circle_buf, line_buf, Color, GraphicsBuffered, St7789vBuffered};
//! # async fn demo<SPI: SpiDevice, DC: OutputPin, RST: OutputPin>(mut display: St7789vBuffered<SPI, DC, RST>) {
//! loop {
//!     display.fill_screen_buf(Color::BLACK);
//!     {
//!         let mut gfx = GraphicsBuffered::new(&mut display);
//!         line_buf(&mut gfx, 0, 0, 239, 319, Color::WHITE);
//!         circle_buf(&mut gfx, 120, 160, 60, Color::CYAN);
//!     } // ← borrow released
//!     display.draw_str_buf(8, 10, b"HELLO", Color::YELLOW, Color::BLACK);
//!     display.flush().await.unwrap();
//! }
//! # }
//! ```
//!
//! ## Note on SPI errors
//!
//! The functions in this crate silently ignore SPI errors.
//! If your application requires detailed error handling, use
//! `display.draw_pixel()` directly.
//! The `*_buf` primitives never access SPI, so SPI errors are not possible.

use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;
pub use hazard3_st7789v::{
    APPROX, Color, DEGREE, DELTA, DIVIDE, GE, INFINITY, LAMBDA, LE, NoPin, PI, PLUS_MINUS,
    SCREEN_H, SCREEN_W, SQRT, St7789v, St7789vBuffered, THETA, TIMES,
};

// ─────────────────────────────────────────────────────────────────────────────
// Graphics context
// ─────────────────────────────────────────────────────────────────────────────

/// Graphics context for the ST7789V 240×320.
///
/// Wraps a `&mut St7789v<SPI, DC, RST>` to:
/// - centralize **clipping** of signed coordinates
/// - provide an async `pixel()` with `i32` coordinates for Bresenham / midpoint algorithms
///
/// The driver retains ownership of the SPI bus and pins.
///
/// # Construction
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789v<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, Graphics, St7789v, line};

/// #     let mut gfx = Graphics::new(&mut display);
///     line(&mut gfx, 0, 0, 239, 319, Color::RED).await;
/// # }
/// ```
///
/// Use [`Graphics::new_no_rst`] if your `St7789v` was constructed
/// with [`St7789v::new_no_rst`].
pub struct Graphics<'a, SPI, DC, RST = NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    display: &'a mut St7789v<SPI, DC, RST>,
}

// ── Constructor without RST ────────────────────────────────────────────────────

impl<'a, SPI, DC> Graphics<'a, SPI, DC, NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    /// Creates a graphics context for an `St7789v` without an RST pin.
    #[inline]
    pub fn new_no_rst(display: &'a mut St7789v<SPI, DC, NoPin>) -> Self {
        Self { display }
    }
}

// ── Constructor with RST ──────────────────────────────────────────────────────

impl<'a, SPI, DC, RST> Graphics<'a, SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Creates a graphics context for an `St7789v` with a hardware RST pin.
    #[inline]
    pub fn new(display: &'a mut St7789v<SPI, DC, RST>) -> Self {
        Self { display }
    }

    /// Draws a pixel with automatic clipping.
    ///
    /// Negative coordinates or coordinates outside `[0, 240)` × `[0, 320)`
    /// are silently ignored: no panic or wrapping.
    ///
    /// SPI errors are ignored (consistent with use in geometric algorithms
    /// that draw thousands of pixels).
    #[inline(always)]
    pub async fn pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && x < SCREEN_W as i32 && y < SCREEN_H as i32 {
            let _ = self.display.draw_pixel(x as u16, y as u16, color).await;
        }
    }

    /// Draws an internal horizontal line (used by fill algorithms).
    ///
    /// Clips vertically and then delegates to `fill_rect`.
    #[inline]
    async fn hline(&mut self, x0: i32, x1: i32, y: i32, color: Color) {
        if y < 0 || y >= SCREEN_H as i32 {
            return;
        }
        let xa = x0.max(0).min(SCREEN_W as i32 - 1) as u16;
        let xb = x1.max(0).min(SCREEN_W as i32 - 1) as u16;
        let (xa, xb) = if xa <= xb { (xa, xb) } else { (xb, xa) };
        let _ = self
            .display
            .fill_rect(xa, y as u16, xb, y as u16, color)
            .await;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bresenham line
// ─────────────────────────────────────────────────────────────────────────────

/// Draws a line between `(x0, y0)` and `(x1, y1)`.
///
/// **Algorithm:** integer-only Bresenham.
/// No floating-point division or multiplication; safe on any MCU without an FPU.
///
/// # Example
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789v<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, Graphics, St7789v, line};

/// #     let mut gfx = Graphics::new(&mut display);
///     line(&mut gfx, 0, 0, 239, 319, Color::WHITE).await;  // full diagonal
///     line(&mut gfx, 0, 0, 239, 319, Color::BLACK).await;  // erase the diagonal
/// # }
/// ```
pub async fn line<SPI, DC, RST>(
    gfx: &mut Graphics<'_, SPI, DC, RST>,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: Color,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        gfx.pixel(x0, y0, color).await;
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Midpoint circle
// ─────────────────────────────────────────────────────────────────────────────

/// Draws the **outline** of a circle.
///
/// **Algorithm:** integer-only midpoint circle.
/// Uses 8-octant symmetry: each iteration draws 8 symmetric pixels,
/// minimizing the number of calls to `pixel()`.
///
/// # Parameters
///
/// - `(cx, cy)`: center
/// - `r`: radius in pixels
/// - `color`: outline color
///
/// # Example
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789v<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, Graphics, St7789v, circle};

/// #     let mut gfx = Graphics::new(&mut display);
///     circle(&mut gfx, 120, 160, 60, Color::CYAN).await;
/// # }
/// ```
pub async fn circle<SPI, DC, RST>(
    gfx: &mut Graphics<'_, SPI, DC, RST>,
    cx: i32,
    cy: i32,
    r: i32,
    color: Color,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    if r <= 0 {
        gfx.pixel(cx, cy, color).await;
        return;
    }
    let mut x = r;
    let mut y = 0;
    let mut err = 0;

    while x >= y {
        gfx.pixel(cx + x, cy + y, color).await;
        gfx.pixel(cx + y, cy + x, color).await;
        gfx.pixel(cx - y, cy + x, color).await;
        gfx.pixel(cx - x, cy + y, color).await;
        gfx.pixel(cx - x, cy - y, color).await;
        gfx.pixel(cx - y, cy - x, color).await;
        gfx.pixel(cx + y, cy - x, color).await;
        gfx.pixel(cx + x, cy - y, color).await;

        y += 1;
        if err <= 0 {
            err += 2 * y + 1;
        } else {
            x -= 1;
            err += 2 * (y - x) + 1;
        }
    }
}

/// **Fills** a circle (solid disk).
///
/// Uses the same midpoint algorithm, but draws horizontal lines
/// between symmetric points on each row.
/// Much faster than drawing pixel by pixel, because `fill_rect`
/// sends data in batches over SPI.
///
/// # Example
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789v<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, Graphics, St7789v, fill_circle};

/// #     let mut gfx = Graphics::new(&mut display);
///     fill_circle(&mut gfx, 120, 160, 50, Color::BLUE).await;
/// # }
/// ```
pub async fn fill_circle<SPI, DC, RST>(
    gfx: &mut Graphics<'_, SPI, DC, RST>,
    cx: i32,
    cy: i32,
    r: i32,
    color: Color,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    if r <= 0 {
        gfx.pixel(cx, cy, color).await;
        return;
    }
    let mut x = r;
    let mut y = 0;
    let mut err = 0;

    while x >= y {
        gfx.hline(cx - x, cx + x, cy + y, color).await;
        gfx.hline(cx - x, cx + x, cy - y, color).await;
        gfx.hline(cx - y, cx + y, cy + x, color).await;
        gfx.hline(cx - y, cx + y, cy - x, color).await;

        y += 1;
        if err <= 0 {
            err += 2 * y + 1;
        } else {
            x -= 1;
            err += 2 * (y - x) + 1;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Triangle
// ─────────────────────────────────────────────────────────────────────────────

/// Draws the **outline** of a triangle defined by three vertices.
///
/// Implemented as three calls to [`line()`], with no additional logic.
///
/// # Example
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789v<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, Graphics, St7789v, triangle};

/// #     let mut gfx = Graphics::new(&mut display);
///     triangle(&mut gfx, 120, 10, 20, 310, 220, 310, Color::GREEN).await;
/// # }
/// ```
pub async fn triangle<SPI, DC, RST>(
    gfx: &mut Graphics<'_, SPI, DC, RST>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: Color,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    line(gfx, x0, y0, x1, y1, color).await;
    line(gfx, x1, y1, x2, y2, color).await;
    line(gfx, x2, y2, x0, y0, color).await;
}

// ─────────────────────────────────────────────────────────────────────────────
// Generalized midpoint ellipse
// ─────────────────────────────────────────────────────────────────────────────

/// Draws the **outline** of an ellipse.
///
/// **Algorithm:** integer-only midpoint ellipse (generalized Bresenham).
/// Two phases: region 1 (slope < -1), then region 2 (slope > -1).
///
/// # Parameters
///
/// - `(cx, cy)`: center
/// - `rx`: horizontal semi-axis in pixels
/// - `ry`: vertical semi-axis in pixels
/// - `color`: outline color
///
/// # Example
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789v<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, Graphics, St7789v, ellipse};

/// #     let mut gfx = Graphics::new(&mut display);
///     ellipse(&mut gfx, 120, 160, 100, 60, Color::MAGENTA).await; // wide ellipse
///     ellipse(&mut gfx, 120, 160, 40, 40, Color::WHITE).await;    // circle (rx == ry)
/// # }
/// ```
pub async fn ellipse<SPI, DC, RST>(
    gfx: &mut Graphics<'_, SPI, DC, RST>,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
    color: Color,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    if rx <= 0 || ry <= 0 {
        gfx.pixel(cx, cy, color).await;
        return;
    }

    let rx2 = rx * rx;
    let ry2 = ry * ry;

    let mut x = 0i32;
    let mut y = ry;

    // Region 1
    let mut d1 = ry2 - rx2 * ry + rx2 / 4;
    let mut dx = 2 * ry2 * x;
    let mut dy = 2 * rx2 * y;

    while dx < dy {
        gfx.pixel(cx + x, cy + y, color).await;
        gfx.pixel(cx - x, cy + y, color).await;
        gfx.pixel(cx + x, cy - y, color).await;
        gfx.pixel(cx - x, cy - y, color).await;

        x += 1;
        dx += 2 * ry2;
        if d1 < 0 {
            d1 += dx + ry2;
        } else {
            y -= 1;
            dy -= 2 * rx2;
            d1 += dx - dy + ry2;
        }
    }

    // Region 2
    let mut d2 = ry2 * (x * x + x) + rx2 * (y * y - 2 * y + 1) - rx2 * ry2 + rx2;

    while y >= 0 {
        gfx.pixel(cx + x, cy + y, color).await;
        gfx.pixel(cx - x, cy + y, color).await;
        gfx.pixel(cx + x, cy - y, color).await;
        gfx.pixel(cx - x, cy - y, color).await;

        y -= 1;
        dy -= 2 * rx2;
        if d2 > 0 {
            d2 += rx2 - dy;
        } else {
            x += 1;
            dx += 2 * ry2;
            d2 += dx - dy + rx2;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// De Casteljau quadratic Bézier curve
// ─────────────────────────────────────────────────────────────────────────────

/// Draws a **quadratic Bézier curve** (3 control points).
///
/// **Algorithm:** integer-only De Casteljau with fixed subdivision.
/// `steps` controls the drawing detail (16–32 is enough for 240×320).
///
/// Interpolation uses integers with ×1024 precision
/// to avoid floating-point calculations.
///
/// # Parameters
///
/// - `(x0, y0)`: starting point
/// - `(x1, y1)`: control point
/// - `(x2, y2)`: endpoint
/// - `steps`: number of segments (recommended: 16 to 32)
/// - `color`: curve color
///
/// # Example
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789v<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, Graphics, St7789v, bezier_quad};

/// #     let mut gfx = Graphics::new(&mut display);
///     // Arch: control point at the top center
///     bezier_quad(&mut gfx, 10, 280, 120, 40, 230, 280, 24, Color::ORANGE).await;
/// # }
/// ```
pub async fn bezier_quad<SPI, DC, RST>(
    gfx: &mut Graphics<'_, SPI, DC, RST>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    steps: i32,
    color: Color,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    if steps <= 0 {
        return;
    }

    let mut px = x0;
    let mut py = y0;

    for i in 1..=steps {
        // t = i / steps in fixed-point ×1024
        let t = (i * 1024) / steps; // t  ∈ [0, 1024]
        let t1 = 1024 - t; // 1-t

        // B(t) = (1-t)²·P0 + 2(1-t)t·P1 + t²·P2  (all ×1024²)
        let nx = (t1 * t1 * x0 + 2 * t1 * t * x1 + t * t * x2) / (1024 * 1024);
        let ny = (t1 * t1 * y0 + 2 * t1 * t * y1 + t * t * y2) / (1024 * 1024);

        line(gfx, px, py, nx, ny, color).await;
        px = nx;
        py = ny;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Filled triangle — scanline
// ─────────────────────────────────────────────────────────────────────────────

/// **Fills** a triangle defined by three vertices.
///
/// **Algorithm:** scanline: sort vertices by Y, then use integer-only
/// linear interpolation of the left and right edges on each horizontal row.
/// Uses `fill_rect` to send each line in a single SPI transfer.
///
/// # Example
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789v<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, Graphics, St7789v, fill_triangle};

/// #     let mut gfx = Graphics::new(&mut display);
///     fill_triangle(&mut gfx, 120, 10, 20, 310, 220, 310, Color::YELLOW).await;
/// # }
/// ```
pub async fn fill_triangle<SPI, DC, RST>(
    gfx: &mut Graphics<'_, SPI, DC, RST>,
    x0: i32,
    mut y0: i32,
    x1: i32,
    mut y1: i32,
    x2: i32,
    mut y2: i32,
    color: Color,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    // Sort vertices by ascending Y (bubble sort on 3 elements)
    let (mut x0, mut x1, mut x2) = (x0, x1, x2);
    if y0 > y1 {
        core::mem::swap(&mut y0, &mut y1);
        core::mem::swap(&mut x0, &mut x1);
    }
    if y1 > y2 {
        core::mem::swap(&mut y1, &mut y2);
        core::mem::swap(&mut x1, &mut x2);
    }
    if y0 > y1 {
        core::mem::swap(&mut y0, &mut y1);
        core::mem::swap(&mut x0, &mut x1);
    }

    let total_h = y2 - y0;
    if total_h == 0 {
        // Degenerate triangle: draw a single line
        let xmin = x0.min(x1).min(x2);
        let xmax = x0.max(x1).max(x2);
        gfx.hline(xmin, xmax, y0, color).await;
        return;
    }

    let upper_h = y1 - y0;
    let lower_h = y2 - y1;

    // Upper half: y0 → y1
    for y in y0..=y1 {
        let dy = y - y0;
        let xa = x0 + (x2 - x0) * dy / total_h;
        let xb = if upper_h == 0 {
            x1
        } else {
            x0 + (x1 - x0) * dy / upper_h
        };
        gfx.hline(xa, xb, y, color).await;
    }

    // Lower half: y1 → y2
    for y in y1..=y2 {
        let dy = y - y0;
        let xa = x0 + (x2 - x0) * dy / total_h;
        let xb = if lower_h == 0 {
            x1
        } else {
            x1 + (x2 - x1) * (y - y1) / lower_h
        };
        gfx.hline(xa, xb, y, color).await;
    }

}

// ─────────────────────────────────────────────────────────────────────────────
// Graphics context for framebuffer
// ─────────────────────────────────────────────────────────────────────────────

/// Graphics context for `St7789vBuffered`.
///
/// Identical to [`Graphics`], but all operations write to the RAM
/// framebuffer via `set_pixel()`. No pixel is sent to the display
/// until [`St7789vBuffered::flush`] is called.
///
/// # Usage pattern
///
/// ```rust,no_run
/// # async fn demo<SPI: embedded_hal_async::spi::SpiDevice, DC: embedded_hal::digital::OutputPin, RST: embedded_hal::digital::OutputPin>(mut display: hazard3_st7789v_graphics::St7789vBuffered<SPI, DC, RST>) {
/// #     use hazard3_st7789v_graphics::{Color, GraphicsBuffered, St7789vBuffered, line_buf, circle_buf};

/// #     let mut gfx = GraphicsBuffered::new(&mut display);
///     line_buf(&mut gfx, 0, 0, 239, 319, Color::WHITE);
///     circle_buf(&mut gfx, 120, 160, 60, Color::CYAN);
///     drop(gfx); // release the borrow
///     display.flush().await.unwrap(); // send the complete frame
/// # }
/// ```
pub struct GraphicsBuffered<'a, SPI, DC, RST = NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    display: &'a mut St7789vBuffered<SPI, DC, RST>,
}

impl<'a, SPI, DC> GraphicsBuffered<'a, SPI, DC, NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    #[inline]
    pub fn new_no_rst(display: &'a mut St7789vBuffered<SPI, DC, NoPin>) -> Self {
        Self { display }
    }
}

impl<'a, SPI, DC, RST> GraphicsBuffered<'a, SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    #[inline]
    pub fn new(display: &'a mut St7789vBuffered<SPI, DC, RST>) -> Self {
        Self { display }
    }

    /// Writes a pixel to the framebuffer with clipping.
    #[inline(always)]
    pub fn pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && x < SCREEN_W as i32 && y < SCREEN_H as i32 {
            self.display.set_pixel(x as u16, y as u16, color);
        }
    }

    /// Horizontal line in the framebuffer (used by fill_circle_buf, fill_triangle_buf).
    #[inline]
    fn hline(&mut self, x0: i32, x1: i32, y: i32, color: Color) {
        if y < 0 || y >= SCREEN_H as i32 { return; }
        let xa = x0.max(0).min(SCREEN_W as i32 - 1) as u16;
        let xb = x1.max(0).min(SCREEN_W as i32 - 1) as u16;
        let (xa, xb) = if xa <= xb { (xa, xb) } else { (xb, xa) };
        self.display.fill_rect_buf(xa, y as u16, xb, y as u16, color);
    }
}


// ─────────────────────────────────────────────────────────────────────────────
// Framebuffer primitives (synchronous)
// ─────────────────────────────────────────────────────────────────────────────

pub fn line_buf<SPI, DC, RST>(
    gfx: &mut GraphicsBuffered<'_, SPI, DC, RST>,
    mut x0: i32, mut y0: i32,
    x1: i32, y1: i32,
    color: Color,
) where SPI: SpiDevice, DC: OutputPin, RST: OutputPin,
{
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        gfx.pixel(x0, y0, color);
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

pub fn circle_buf<SPI, DC, RST>(
    gfx: &mut GraphicsBuffered<'_, SPI, DC, RST>,
    cx: i32, cy: i32, r: i32, color: Color,
) where SPI: SpiDevice, DC: OutputPin, RST: OutputPin,
{
    if r <= 0 { gfx.pixel(cx, cy, color); return; }
    let mut x = r; let mut y = 0; let mut err = 0;
    while x >= y {
        gfx.pixel(cx + x, cy + y, color); gfx.pixel(cx + y, cy + x, color);
        gfx.pixel(cx - y, cy + x, color); gfx.pixel(cx - x, cy + y, color);
        gfx.pixel(cx - x, cy - y, color); gfx.pixel(cx - y, cy - x, color);
        gfx.pixel(cx + y, cy - x, color); gfx.pixel(cx + x, cy - y, color);
        y += 1;
        if err <= 0 { err += 2 * y + 1; }
        else { x -= 1; err += 2 * (y - x) + 1; }
    }
}

pub fn fill_circle_buf<SPI, DC, RST>(
    gfx: &mut GraphicsBuffered<'_, SPI, DC, RST>,
    cx: i32, cy: i32, r: i32, color: Color,
) where SPI: SpiDevice, DC: OutputPin, RST: OutputPin,
{
    if r <= 0 { gfx.pixel(cx, cy, color); return; }
    let mut x = r; let mut y = 0; let mut err = 0;
    while x >= y {
        gfx.hline(cx - x, cx + x, cy + y, color);
        gfx.hline(cx - x, cx + x, cy - y, color);
        gfx.hline(cx - y, cx + y, cy + x, color);
        gfx.hline(cx - y, cx + y, cy - x, color);
        y += 1;
        if err <= 0 { err += 2 * y + 1; }
        else { x -= 1; err += 2 * (y - x) + 1; }
    }
}

pub fn triangle_buf<SPI, DC, RST>(
    gfx: &mut GraphicsBuffered<'_, SPI, DC, RST>,
    x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: Color,
) where SPI: SpiDevice, DC: OutputPin, RST: OutputPin,
{
    line_buf(gfx, x0, y0, x1, y1, color);
    line_buf(gfx, x1, y1, x2, y2, color);
    line_buf(gfx, x2, y2, x0, y0, color);
}

pub fn fill_triangle_buf<SPI, DC, RST>(
    gfx: &mut GraphicsBuffered<'_, SPI, DC, RST>,
    x0: i32, mut y0: i32,
    x1: i32, mut y1: i32,
    x2: i32, mut y2: i32,
    color: Color,
) where SPI: SpiDevice, DC: OutputPin, RST: OutputPin,
{
    let (mut x0, mut x1, mut x2) = (x0, x1, x2);
    if y0 > y1 { core::mem::swap(&mut y0, &mut y1); core::mem::swap(&mut x0, &mut x1); }
    if y1 > y2 { core::mem::swap(&mut y1, &mut y2); core::mem::swap(&mut x1, &mut x2); }
    if y0 > y1 { core::mem::swap(&mut y0, &mut y1); core::mem::swap(&mut x0, &mut x1); }
    let total_h = y2 - y0;
    if total_h == 0 {
        gfx.hline(x0.min(x1).min(x2), x0.max(x1).max(x2), y0, color);
        return;
    }
    let upper_h = y1 - y0;
    let lower_h = y2 - y1;
    for y in y0..=y1 {
        let dy = y - y0;
        let xa = x0 + (x2 - x0) * dy / total_h;
        let xb = if upper_h == 0 { x1 } else { x0 + (x1 - x0) * dy / upper_h };
        gfx.hline(xa, xb, y, color);
    }
    for y in y1..=y2 {
        let dy = y - y0;
        let xa = x0 + (x2 - x0) * dy / total_h;
        let xb = if lower_h == 0 { x1 } else { x1 + (x2 - x1) * (y - y1) / lower_h };
        gfx.hline(xa, xb, y, color);
    }
}

pub fn ellipse_buf<SPI, DC, RST>(
    gfx: &mut GraphicsBuffered<'_, SPI, DC, RST>,
    cx: i32, cy: i32, rx: i32, ry: i32, color: Color,
) where SPI: SpiDevice, DC: OutputPin, RST: OutputPin,
{
    if rx <= 0 || ry <= 0 { gfx.pixel(cx, cy, color); return; }
    let rx2 = rx * rx; let ry2 = ry * ry;
    let mut x = 0i32; let mut y = ry;
    let mut d1 = ry2 - rx2 * ry + rx2 / 4;
    let mut dx = 2 * ry2 * x; let mut dy = 2 * rx2 * y;
    while dx < dy {
        gfx.pixel(cx + x, cy + y, color); gfx.pixel(cx - x, cy + y, color);
        gfx.pixel(cx + x, cy - y, color); gfx.pixel(cx - x, cy - y, color);
        x += 1; dx += 2 * ry2;
        if d1 < 0 { d1 += dx + ry2; }
        else { y -= 1; dy -= 2 * rx2; d1 += dx - dy + ry2; }
    }
    let mut d2 = ry2 * (x * x + x) + rx2 * (y * y - 2 * y + 1) - rx2 * ry2 + rx2;
    while y >= 0 {
        gfx.pixel(cx + x, cy + y, color); gfx.pixel(cx - x, cy + y, color);
        gfx.pixel(cx + x, cy - y, color); gfx.pixel(cx - x, cy - y, color);
        y -= 1; dy -= 2 * rx2;
        if d2 > 0 { d2 += rx2 - dy; }
        else { x += 1; dx += 2 * ry2; d2 += dx - dy + rx2; }
    }
}

pub fn bezier_quad_buf<SPI, DC, RST>(
    gfx: &mut GraphicsBuffered<'_, SPI, DC, RST>,
    x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32,
    steps: i32, color: Color,
) where SPI: SpiDevice, DC: OutputPin, RST: OutputPin,
{
    if steps <= 0 { return; }
    let mut px = x0; let mut py = y0;
    for i in 1..=steps {
        let t = (i * 1024) / steps;
        let t1 = 1024 - t;
        let nx = (t1 * t1 * x0 + 2 * t1 * t * x1 + t * t * x2) / (1024 * 1024);
        let ny = (t1 * t1 * y0 + 2 * t1 * t * y1 + t * t * y2) / (1024 * 1024);
        line_buf(gfx, px, py, nx, ny, color);
        px = nx; py = ny;
    }
}
