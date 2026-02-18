# Fractal Oxide

Interactive fractal explorer built with Rust, eframe/egui, and Rayon for parallel CPU rendering.

![Fractal Oxide Screenshot](images/screenshot.png)

## Features

### Key Features
- **Supersampling** - Anti-aliased oversampling for high-quality visualizations and output
- **High-Performance** - CPU-parallel using all available cores
- **Optimized** - Optimized previews, zooming and panning, with pixel caching
- **Configurable** - Variety of colormaps and color processors
- **Adaptive** - Varied iteration limits while zooming to preserve detail

### Fractal Types
- **Mandelbrot** - Classic Mandelbrot set with adjustable power
- **Julia** - Julia set with customizable c_real, c_imag parameters  
- **Burning Ship** - Iterated with absolute values
- **Tricorn** - Also known as Mandelbar (conjugated iteration)
- **Celtic** - Variant with absolute value on real component of z^2
- **Newton** - Newton's method visualization for z^3 - 1 = 0
- **Biomorph** - Classical Pickover biomorph (z^n+c with |Re|/|Im| escape test)
- **Phoenix** - Ushiki Phoenix with memory term (c=0.5667, p=-0.5)
- **Multibrot** - Mandelbrot generalized to arbitrary power (delegates to Mandelbrot engine)
- **Spider** - Classical Spider with evolving c parameter (z=z^2+c, c=c/2+z)
- **Orbit Trap** - Mandelbrot variant tracking minimum distance to a trap point (default: origin)
- **Pickover Stalk** - Mandelbrot variant creating organic stalk patterns near axes


## Building

Building, running and testing:

```bash
make build && make run
# or
cargo build --release && cargo run --release

# Debug mode with logging
RUST_LOG=debug cargo run

# Run tests
cargo test
make test

# Format and lint
cargo fmt
cargo clippy -- -D warnings
make fmt
make lint
```

## Distribution

Create platform-specific installers:

```bash
# Create macOS .dmg package
make dist-mac

# Create Linux .deb package (for apt install)
make dist-linux

# Create Windows installer package
make dist-windows

# Create all distributions
make dist
```

**Distribution outputs:**
- macOS: `dist/FractalOxide-vX.Y.Z-macOS.dmg`
- Linux: `dist/fractal-oxide_vX.Y.Z_amd64.deb`
- Windows: `dist/FractalOxide-vX.Y.Z-windows.zip`

### Installing on macOS
```bash
# Open the .dmg and drag to Applications
make dist-mac
open dist/FractalOxide-0.1.0-macOS.dmg
```

### Installing on Linux (Debian/Ubuntu)
```bash
make dist-linux
sudo apt install ./dist/fractal-oxide_0.1.0_amd64.deb
# Run with: fractal-oxide
```

### Installing on Windows
```bash
make dist-windows
# Extract FractalOxide-0.1.0-windows.zip
# Run install.bat as Administrator
# Or manually copy FractalOxide.exe to your preferred location
```


## Dependencies

- `eframe` / `egui` - GUI framework
- `rayon` - Data-parallel processing for rendering
- `image` - PNG export functionality
- `num-complex` - Complex number type for orbit data
- `serde` / `serde_json` - Configuration serialization
- `dirs` - Cross-platform config directory detection



## License

- [MIT License](LICENSE)
