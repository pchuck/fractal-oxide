# Fractal Oxide

## Detailed Features and  Specification

### [Key Features](README.md#key-features)

### [Fractal Types](README.md#fractal-Types)

### Display Features
- **Mouse Coordinates** - Shows fractal coordinates under cursor
- **Zoom Preview** - Preview when zooming for instant feedback
- **Progress Bar** - Shows rendering progress for large images, with detailed render status
- **Supersampling** - 2x supersampling for smoother edges
- **Mini-map** - Overview showing current view rectangle within larger fractal set
- **Pan Optimization** - Pre-calculated pixels are cached for higher performance panning
- **Texture Caching** - GPU texture only recreated on image changes
- **About Dialog** - Shows App info with cached image and copyright

### Smart Features
- **Adaptive Iterations** - Varied iteration limits while zooming to preserve detail
- **Anti-Aliasing** - Supersampling option for smoother edges
- **Efficient Panning** - Arrow key panning reuses existing pixel data, only rendering new edge regions
- **Power=2 Fast Path** - All De Moivre-based fractals use direct algebraic formula when power=2 (3-5x faster)

### Color Palettes
- **Classic** - Rainbow gradient (black -> blue -> cyan -> green -> yellow -> red -> white)
- **Fire** - Heat map (black -> red -> orange -> yellow -> white)
- **Ice** - Cold tones (black -> blue -> cyan -> white)
- **Grayscale** - Black to white gradient
- **Psychedelic** - HSV cycling with adjustable offset

### Color Processors
Color processors transform fractal iteration data into colors using different algorithms.
All processors receive full orbit data (final_z, orbit distances) via `compute_full()`:
- **Standard Palette** - Direct palette mapping based on iteration count
- **Smooth Coloring** - Continuous coloring using logarithmic smoothing for gradient bands
- **Orbit Trap (Real Axis)** - Traps orbits near the real axis (y=0) for stalk-like patterns
- **Orbit Trap (Imaginary Axis)** - Traps orbits near the imaginary axis (x=0) for organic patterns
- **Orbit Trap (Origin)** - Traps orbits near the origin for center-focused patterns


### State Management
- **Per-Fractal State** - Each fractal remembers its view position, zoom, iterations, palette, color processor, and parameters
- **Per-Fractal Undo/Redo** - Separate 50-step command history for each fractal type
- **Bookmarks** - Save interesting locations with names, including position, zoom, iterations, palette, color processor, and all fractal parameters
- **Configuration File** - Saves actual window size, defaults, bookmarks, and settings to `~/.config/fractal-oxide/config.json`

### Interactive Controls
- **Click + Drag** - Select zoom region
- **Mouse Wheel** - Zoom in/out at cursor position (focus-preserving)
- **Arrow Keys** - Pan view with pixel reuse optimization (reuses rendered pixels)
- **+ / -** - Zoom in/out by 1.5x
- **R** - Reset view to defaults
- **Shift+R** - Reset all settings (view, palette, parameters)
- **S** - Save image (1x, 2x and 4x resolutions)
- **Ctrl+Z** - Undo last view change
- **Ctrl+Y** - Redo view change


### Export Options
- **Save (S) with Radio Buttons** - Select 1x, 2x, or 4x resolution, then click Save
- **High Resolution** - 2x and 4x renders at higher resolution for better quality
- **Color Processor** - Exports use the current color processor (not just palette)
- **Supersampling** - 2x internal render with box filter downsampling
- All exports saved to `images/` directory with auto-generated filenames

## UI Layout

### Control Panel (Left Side)
- **Fractal Type** | **Render Status** - Side by side with vertical separator
  - Fractal dropdown on left
  - Thread count and render time on right
- **Color Palette** | **Color Processor** - Side by side with vertical separator
  - Palette dropdown on left
  - Color processor dropdown on right
- **Iterations** - Slider for max iterations
- **Fractal Parameters** - Dynamic controls based on fractal type
- **Save (S)** - Button with 1x/2x/4x radio buttons inline
- **Undo (^Z)** | **Redo (^Y)** - Side by side
- **Bookmarks** - List with Add button, status messages shown here
- **Settings** - Supersampling, Adaptive Iterations, Minimap toggles
- **Mouse** | **Keyboard** - Input reference
- **About** - Opens About dialog with image and copyright

### Display Panel (Center)
- Main fractal view
- Minimap overlay (top-right, when enabled)
- Selection rectangle (when dragging)

## Parameters

### Fractal-Specific
- **Power** (Mandelbrot, Burning Ship, Tricorn, Celtic, Multibrot) - Exponent value (1.0-10.0)
- **c_real / c_imag** (Julia, Phoenix) - Fractal constant (-2.0 to 2.0)
- **Memory** (Phoenix) - Memory coefficient creating phoenix patterns (-1.0 to 1.0), default -0.5
- **Default Phoenix**: c_real=0.5667, c_imag=0.0, memory=-0.5 (classic Ushiki Phoenix)
- **Power / Escape Radius** (Biomorph) - Power (2.0-8.0, default 3.0) and biomorph escape test radius (2.0-100.0, default 10.0)
- **Tolerance** (Newton) - Root detection sensitivity (0.0001-0.1)
- **trap_x / trap_y** (Orbit Trap) - Trap point coordinates (-2.0 to 2.0), default origin (0, 0)
- **thickness / intensity** (Pickover Stalk) - Stalk thickness (0.01-1.0) and intensity (1.0-100.0)
- **Spider** - No parameters (uses classical z=z^2+c, c=c/2+z algorithm)

### Global
- **Iterations** - Maximum iteration count (16-2000)
- **Color Offset** (Psychedelic palette) - Color rotation (0.0-1.0)
- **Supersampling** - Enable 2x supersampling for smoother edges
- **Adaptive Iterations** - Auto-adjust iterations based on zoom level

## Bookmarks

Save interesting locations for later:
- Click "Bookmark" button to save current view
- Each bookmark saves: name, fractal type, position, zoom, iterations, palette, color processor, and all fractal parameters
- Click bookmark name to restore that view (including fractal-specific params like Julia c values)
- Bookmarks persist across sessions in config file

## Configuration

Settings are automatically saved to:
- **Linux**: `~/.config/fractal-oxide/config.json`
- **macOS**: `~/Library/Application Support/fractal-oxide/config.json`
- **Windows**: `%APPDATA%\fractal-oxide\config.json`

Saved settings include:
- Actual window size (tracked each frame, saved on exit)
- Default fractal type and palette
- Default iteration count
- Supersampling preference
- Adaptive iterations setting
- All bookmarks (with full fractal state)

## Project Layout

```
src/
├── main.rs              # Application entry, FractalApp with RenderState + InteractionState
├── ui/mod.rs            # Control panel UI components
├── fractal/mod.rs       # Fractal trait, compute_full(), & 12 implementations
├── fractal/registry.rs  # Fractal factory and registry
├── palette/mod.rs       # Color palette system (5 palettes)
├── color_pipeline.rs    # Color processor system (5 processors, FractalResult, OrbitData)
├── command.rs           # Command pattern for undo/redo (uses FractalViewState)
├── renderer/mod.rs      # Rendering engine with pan optimization
└── viewport.rs          # Viewport and coordinate transforms
```

### Key Components

**FractalApp**: Main application state composed of:
- `RenderState` - Rendering engine, config, progress, caches, texture handles
- `InteractionState` - Drag state, zoom preview, mouse position, status messages
- Per-fractal views, command histories, bookmarks, viewport

**Fractal trait**: Provides `compute()` for iteration count and `compute_full()` for rich `FractalResult` with orbit data and final_z, used by all color processors.

**Multibrot**: Delegates to Mandelbrot's compute methods (same formula, different default power/range).

**Named constants**: All magic numbers extracted to `const` declarations at module level (`BAILOUT_R2`, `POWER2_EPSILON`, `UNDO_HISTORY_CAPACITY`, `MINIMAP_SIZE`, etc.).

## Test Coverage

| Module | Coverage |
|---|---|
| `fractal/mod.rs` | All fractal types, compute_full, edge cases, parameter clamping |
| `palette/mod.rs` | All palettes, HSV conversion, interpolation, offsets |
| `command.rs` | View/parameter commands, history, undo/redo, limits |
| `color_pipeline.rs` | All processors, orbit data, smooth coloring |
| `renderer/mod.rs` | Screen-to-fractal mapping, pan regions, downsampling |
| `viewport.rs` | Screen-to-world, pan, zoom, roundtrip |
| `fractal/registry.rs` | Registry, factory, metadata |

## License

- [MIT License](LICENSE)
