# Agent Guidelines - Rust Fractal Oxide

## Project Overview

Interactive fractal explorer built with Rust, eframe/egui, and Rayon for parallel CPU rendering. Supports 12 fractal types with 5 color palettes, 5 color processors, per-fractal state persistence, bookmarks, undo/redo history, pan optimization with pixel reuse, scroll-wheel zoom, and thread count display.

## Build and Run Commands

```bash
# Build and run
make build && make run
cargo build --release && cargo run --release

# Debug mode with logging
RUST_LOG=debug cargo run

# Run tests
cargo test
make test

# Run single test
cargo test test_name -- --nocapture
cargo test fractal::tests::test_mandelbrot_center -- --nocapture

# Tests with output
cargo test -- --show-output

# Format and lint
cargo fmt
cargo clippy -- -D warnings

# Using Makefile
make fmt          # Format code
make lint         # Run clippy
make test-show    # Run tests with output
make clean        # Clean build artifacts
make deps         # Update dependencies
```

## Distribution Commands

```bash
# Build all distribution packages
make dist

# Platform-specific distributions
make dist-mac      # Creates dist/FractalExplorer-0.1.0-macOS.dmg
make dist-linux    # Creates dist/fractal-explorer_0.1.0_amd64.deb
make dist-windows  # Creates dist/FractalExplorer-0.1.0-windows.zip

# Clean distribution files
make dist-clean
```

**Distribution Details:**
- **macOS (.dmg)**: Creates .app bundle with Info.plist, ready for distribution
- **Linux (.deb)**: Debian package with desktop entry, installable via apt
- **Windows (.zip)**: Portable ZIP with executable and install.bat script

## Code Style

### Rust Conventions

1. **Imports**: Absolute paths for external crates (`use image::ImageBuffer`), relative for internal (`use crate::fractal::Fractal`)
2. **Formatting**: Follow rustfmt defaults (4 spaces, 100 char line length)
3. **Naming**: PascalCase types (`FractalRenderer`), snake_case functions (`compute_iteration`), UPPER_SNAKE constants (`BAILOUT_R2`, `MINIMAP_SIZE`)
4. **Types**: Explicit in public APIs; use `i32`/`f32`/`f64`, `u32` for counts, `Vec<T>` for dynamic arrays, `HashMap` for parameters
5. **Constants**: All magic numbers must be named constants at module level. See `main.rs` constants block and `fractal/mod.rs` for `BAILOUT_R2`/`POWER2_EPSILON`.

### Error Handling

- Use `Result<T, E>` for fallible operations
- Prefer `anyhow` for app-level, `thiserror` for library errors
- Never panic in library code; return errors instead

```rust
pub fn save_config(&self) -> Result<(), String> {
    let path = Self::config_path().ok_or("Could not determine config directory")?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write: {}", e))?;
    Ok(())
}
```

### Documentation

- Document public APIs with doc comments (`///`)
- Include examples where helpful
- Document what `compute()` returns (0 to max_iter, where max_iter means inside set)
- Document fractal formulae accurately against known mathematical standards

## Module Structure

```
src/
├── main.rs              # Entry point, FractalApp (RenderState + InteractionState), constants
├── ui/mod.rs            # Control panel, fractal/palette/processor dropdowns
├── fractal/mod.rs       # Fractal trait + 12 implementations + BAILOUT_R2/POWER2_EPSILON
├── fractal/registry.rs  # Fractal factory and registry
├── palette/mod.rs       # Color palette system (5 palettes)
├── color_pipeline.rs    # Color processor system (5 processors), FractalResult, OrbitData
├── command.rs           # Command pattern for undo/redo (uses FractalViewState directly)
├── renderer/mod.rs      # Rendering engine with pan optimization
└── viewport.rs          # Viewport and coordinate transforms
```

## Core Data Structures

**FractalApp** (decomposed into sub-structs):
```rust
struct FractalApp {
    fractal: Box<dyn Fractal>,
    controls: FractalControls,
    views: HashMap<FractalType, FractalViewState>,
    command_histories: HashMap<FractalType, CommandHistory>,
    render: RenderState,       // Rendering engine, caches, progress, textures
    interaction: InteractionState, // Drag, zoom preview, mouse pos, status
    bookmarks: Vec<Bookmark>,
    // ... minimap, about dialog, viewport, registry
}
```

**RenderState** (rendering concerns):
```rust
struct RenderState {
    engine: RenderEngine,
    config: Option<RenderConfig>,
    needs_render: bool,
    is_rendering: bool,
    render_progress: f32,
    cached_image: Option<egui::ColorImage>,
    cached_texture: Option<egui::TextureHandle>,
    texture_dirty: bool,
    // ... timing, chunks, partial regions, supersampling, adaptive
}
```

**InteractionState** (input concerns):
```rust
#[derive(Default)]
struct InteractionState {
    drag_start: Option<egui::Pos2>,
    drag_current: Option<egui::Pos2>,
    zoom_preview: Option<ZoomPreview>,
    mouse_fractal_pos: Option<(f64, f64)>,
    status_message: Option<(String, Instant)>,
}
```

**FractalViewState** (canonical per-fractal state, used by AppState for undo/redo):
```rust
pub struct FractalViewState {
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
    pub max_iterations: u32,
    pub fractal_params: HashMap<String, f64>,
    pub palette_type: PaletteType,
    pub color_processor_type: ColorProcessorType,
}
```

**Fractal Trait:**
```rust
pub trait Fractal: Send + Sync {
    fn name(&self) -> &str;
    fn parameters(&self) -> Vec<Parameter>;
    fn set_parameter(&mut self, name: &str, value: f64);
    fn get_parameter(&self, name: &str) -> Option<f64>;
    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32;
    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult; // orbit data + final_z
}
```

**ColorProcessorType:**
- `Palette` - Direct palette mapping
- `Smooth` - Logarithmic smoothing for continuous bands
- `OrbitTrapReal` - Traps orbits near real axis
- `OrbitTrapImag` - Traps orbits near imaginary axis
- `OrbitTrapOrigin` - Traps orbits near origin

## Adding New Fractals

1. Add variant to `FractalType` enum in `src/fractal/mod.rs`
2. Implement the `Fractal` trait with `compute()` and `compute_full()` methods
3. Add factory struct in `src/fractal/registry.rs`
4. Add `default_center()` entry for the fractal type
5. Register in `FractalRegistry::register_defaults()`

For simple power-based fractals, use the `impl_power_fractal!` macro. For fractals sharing compute logic (like Multibrot), delegate to an existing implementation.

## Fractal Formulae Reference

| Fractal | Formula | Notes |
|---|---|---|
| Mandelbrot | z = z^d + c, z0=0 | Power=2 uses fast algebraic path |
| Julia | z = z^d + c, z0=pixel | Fixed c, variable z0 |
| Burning Ship | z = (\|Re(z)\| + i\|Im(z)\|)^d + c | abs before squaring |
| Tricorn | z = conj(z)^d + c | Conjugation: -2ab in imaginary |
| Celtic | new_re = \|Re(z^2)\| + c_re | abs on real part only, no conjugation |
| Newton | z = z - (z^3-1)/(3z^2) | Convergence to cube roots of unity |
| Biomorph | z = z^d + c, biomorph test | Inside if \|Re(z)\| < R or \|Im(z)\| < R |
| Phoenix | z = z^2 + c + p*z_{n-1} | Ushiki: c=0.5667, p=-0.5 |
| Multibrot | z = z^d + c (delegates to Mandelbrot) | Default power=3 |
| Spider | z = z^2 + c, c = c/2 + z | Both z and c evolve |
| Orbit Trap | z = z^2 + c, track min dist | Default trap at origin |
| Pickover Stalk | z = z^2 + c (z0=c), track axis dist | z0=pixel, not z0=0 |

## Per-Fractal State

Each fractal type maintains its own state in `views: HashMap<FractalType, FractalViewState>`:
- View position (center_x, center_y, zoom)
- Iteration count (max_iterations)
- Fractal-specific parameters (e.g., power, c_real, c_imag, memory)
- Selected palette and color processor

When switching fractals, the app automatically restores the previous state for that fractal type, including the color processor.

## Image Export

- Save button exports to `images/{fractal}_{palette}_{width}x{height}.png`
- Supports 1x, 2x, and 4x resolution exports
- Exports use the current color processor (not just palette)
- Option for supersampling (2x internal render with box filter)

## Testing (78 tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mandelbrot_center() {
        let m = Mandelbrot::default();
        let result = m.compute(-0.5, 0.0, 100);
        assert!(result >= 100, "Center should not escape");
    }
}
```

Test coverage spans all 12 fractals, all 5 palettes, HSV conversion, color processors, orbit data, command history, viewport transforms, renderer, and edge cases (parameter clamping, max_iter=1, Newton singularity).

## Git Workflow

- Commit format: `type: description` (e.g., `feat: add Phoenix fractal`)
- Types: `feat`, `fix`, `refactor`, `docs`, `test`
- Run `cargo fmt` and `cargo clippy` before committing

## Common Tasks

- **Add UI parameter**: Add to fractal's `parameters()` -> saved in `FractalViewState.fractal_params` -> restored on switch
- **Modify fractal computation**: Edit `compute()` and `compute_full()` methods. Ensure both return consistent iteration counts.
- **Performance**: Use Rayon parallel iterators (`par_iter()`), chunked rendering for progress updates. Add `if (power - 2.0).abs() < POWER2_EPSILON` fast path for power-based fractals.
- **Pan optimization**: Arrow key panning shifts pixels and only renders edge regions (see `pan_view()` in main.rs)
- **Bookmarks**: Saved in AppConfig with full fractal state (params, color processor). Persist to `~/.config/fractal-explorer/config.json`
- **Color Processor**: Add variant to `ColorProcessorType` -> implement `ColorProcessor` trait -> add to factory method. Processors receive `FractalResult` with orbit data.
- **Thread count display**: Uses `rayon::current_num_threads()` in `RenderStatus` struct
- **New constants**: Add to the constants block at the top of `main.rs` or fractal-specific constants in `fractal/mod.rs`
- **Render state**: Access via `self.render.*` (engine, caches, progress, flags)
- **Interaction state**: Access via `self.interaction.*` (drag, mouse, status)

---

**Last updated**: 2026-02-13 (12 fractals, 5 color processors, 78 tests, decomposed FractalApp, named constants, scroll-wheel zoom, classical Spider/Biomorph/Phoenix)
