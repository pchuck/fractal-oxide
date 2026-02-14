use num_complex::Complex64;
use serde::{Deserialize, Serialize};

use crate::color_pipeline::{FractalResult, OrbitData};

pub mod registry;

/// Squared bailout radius: escape when |z|^2 > BAILOUT_R2
const BAILOUT_R2: f64 = 4.0;
/// Epsilon for power=2 fast path comparison
const POWER2_EPSILON: f64 = 1e-10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FractalType {
    #[default]
    Mandelbrot,
    Julia,
    BurningShip,
    Tricorn,
    Celtic,
    Newton,
    Biomorph,
    Phoenix,
    Multibrot,
    Spider,
    OrbitTrap,
    PickoverStalk,
}

impl FractalType {
    /// Returns the default view center for this fractal type.
    /// Used to position the camera when first viewing the fractal.
    pub fn default_center(&self) -> (f64, f64) {
        match self {
            FractalType::Mandelbrot => (-0.5, 0.0),
            FractalType::Julia => (0.0, 0.0),
            FractalType::BurningShip => (-0.5, -0.5),
            FractalType::Tricorn => (0.0, 0.0),
            FractalType::Celtic => (0.0, 0.0),
            FractalType::Newton => (0.0, 0.0),
            FractalType::Biomorph => (0.0, 0.0),
            FractalType::Phoenix => (0.0, 0.0),
            FractalType::Multibrot => (0.0, 0.0),
            FractalType::Spider => (0.0, 0.0),
            FractalType::OrbitTrap => (-0.5, 0.0),
            FractalType::PickoverStalk => (-0.5, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
}

/// Trait for fractal implementations.
///
/// Each fractal provides:
/// - A name for display purposes
/// - Configurable parameters with min/max bounds
/// - A computation function that returns iteration count
pub trait Fractal: Send + Sync {
    /// Returns the display name of this fractal.
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Returns the list of configurable parameters for this fractal.
    fn parameters(&self) -> Vec<Parameter>;

    /// Sets a parameter value, clamping to valid range.
    fn set_parameter(&mut self, name: &str, value: f64);

    /// Gets the current value of a parameter, or None if not found.
    #[allow(dead_code)]
    fn get_parameter(&self, name: &str) -> Option<f64>;

    /// Computes the iteration count for a point in the fractal.
    ///
    /// Returns a value from 0 to max_iter:
    /// - max_iter: Point is inside the set (did not escape)
    /// - lower values: Point escaped after that many iterations
    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32;

    /// Computes the full fractal result including orbit data and final z value.
    ///
    /// The default implementation wraps `compute()` but does not provide
    /// orbit data or final_z. Override this in fractal implementations
    /// to provide rich data for smooth coloring and orbit trap processors.
    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let iterations = self.compute(cx, cy, max_iter);
        if iterations >= max_iter {
            FractalResult::inside_set(iterations)
        } else {
            FractalResult::escaped(iterations, Complex64::new(0.0, 0.0), OrbitData::default())
        }
    }
}

/// Macro to generate Fractal implementations for simple power-based fractals.
///
/// Generates:
/// - name() method
/// - parameters() returning a single "power" parameter
/// - set_parameter() and get_parameter() implementations
///
/// Usage: impl_power_fractal!(StructName, "Display Name")
macro_rules! impl_power_fractal {
    ($struct_name:ident, $display_name:expr) => {
        impl Fractal for $struct_name {
            fn name(&self) -> &str {
                $display_name
            }

            fn parameters(&self) -> Vec<Parameter> {
                vec![Parameter {
                    name: "power".to_string(),
                    value: self.power,
                    min: 1.0,
                    max: 8.0,
                }]
            }

            fn set_parameter(&mut self, name: &str, value: f64) {
                if name == "power" {
                    self.power = value.clamp(1.0, 8.0);
                }
            }

            fn get_parameter(&self, name: &str) -> Option<f64> {
                match name {
                    "power" => Some(self.power),
                    _ => None,
                }
            }

            fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
                self.compute_point(cx, cy, max_iter)
            }

            fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
                self.compute_point_full(cx, cy, max_iter)
            }
        }
    };
}

// ============================================================================
// Mandelbrot Set
// ============================================================================

/// The classic Mandelbrot set.
///
/// Defined by the iteration: z_{n+1} = z_n^power + c
/// where c is the complex coordinate (cx, cy).
///
/// The set consists of all points where the iteration does not diverge
/// (remain bounded) as n -> infinity.
pub struct Mandelbrot {
    pub power: f64,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Mandelbrot { power: 2.0 }
    }
}

impl Mandelbrot {
    /// Computes iterations for a single point using De Moivre's theorem
    /// for arbitrary power exponentiation.
    fn compute_point(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return i;
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                // Fast path for power=2: use direct algebraic formula
                let new_re = r2 - i2 + c_re;
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                // Use De Moivre's theorem: (r*e^(iθ))^power = r^power * e^(i*power*θ)
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }
        }

        max_iter
    }

    /// Full computation with orbit data for color processors
    fn compute_point_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;
        let mut orbit_data = OrbitData::new();

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return FractalResult::escaped(i, Complex64::new(z_re, z_im), orbit_data);
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = r2 - i2 + c_re;
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

impl_power_fractal!(Mandelbrot, "Mandelbrot");

// ============================================================================
// Julia Set
// ============================================================================

/// Julia sets for the function f(z) = z^power + c.
///
/// Unlike Mandelbrot where c varies per pixel, Julia uses a fixed c value
/// and varies the initial z (which corresponds to the pixel coordinate).
///
/// Each c value produces a different Julia set. Famous examples:
/// - c = -0.7 + 0.27015i (default): Douady rabbit
/// - c = -0.123 + 0.745i: Douady rabbit variant
/// - c = -0.391 - 0.587i: San Marco fractal
pub struct Julia {
    pub c_real: f64,
    pub c_imag: f64,
    pub power: f64,
}

impl Default for Julia {
    fn default() -> Self {
        Julia {
            c_real: -0.7,
            c_imag: 0.27015,
            power: 2.0,
        }
    }
}

impl Fractal for Julia {
    fn name(&self) -> &str {
        "Julia"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "c_real".to_string(),
                value: self.c_real,
                min: -2.0,
                max: 2.0,
            },
            Parameter {
                name: "c_imag".to_string(),
                value: self.c_imag,
                min: -2.0,
                max: 2.0,
            },
            Parameter {
                name: "power".to_string(),
                value: self.power,
                min: 1.0,
                max: 8.0,
            },
        ]
    }

    fn set_parameter(&mut self, name: &str, value: f64) {
        match name {
            "c_real" => self.c_real = value.clamp(-2.0, 2.0),
            "c_imag" => self.c_imag = value.clamp(-2.0, 2.0),
            "power" => self.power = value.clamp(1.0, 8.0),
            _ => {}
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "c_real" => Some(self.c_real),
            "c_imag" => Some(self.c_imag),
            "power" => Some(self.power),
            _ => None,
        }
    }

    fn compute(&self, zx: f64, zy: f64, max_iter: u32) -> u32 {
        let mut z_re = zx;
        let mut z_im = zy;
        let c_re = self.c_real;
        let c_im = self.c_imag;
        let power = self.power;

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return i;
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = r2 - i2 + c_re;
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }
        }

        max_iter
    }

    fn compute_full(&self, zx: f64, zy: f64, max_iter: u32) -> FractalResult {
        let mut z_re = zx;
        let mut z_im = zy;
        let c_re = self.c_real;
        let c_im = self.c_imag;
        let power = self.power;
        let mut orbit_data = OrbitData::new();

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return FractalResult::escaped(i, Complex64::new(z_re, z_im), orbit_data);
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = r2 - i2 + c_re;
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

// ============================================================================
// Burning Ship
// ============================================================================

/// The Burning Ship fractal.
///
/// Similar to Mandelbrot but with absolute values:
/// z_{n+1} = (|Re(z_n)| + i*|Im(z_n)|)^power + c
///
/// Creates a distinctive "burning ship" appearance in the negative quadrant.
pub struct BurningShip {
    pub power: f64,
}

impl Default for BurningShip {
    fn default() -> Self {
        BurningShip { power: 2.0 }
    }
}

impl BurningShip {
    fn compute_point(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return i;
            }

            // Burning Ship: apply abs BEFORE power transformation
            z_re = z_re.abs();
            z_im = z_im.abs();

            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = r2 - i2 + c_re;
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }
        }

        max_iter
    }

    fn compute_point_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;
        let mut orbit_data = OrbitData::new();

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return FractalResult::escaped(i, Complex64::new(z_re, z_im), orbit_data);
            }

            z_re = z_re.abs();
            z_im = z_im.abs();

            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = r2 - i2 + c_re;
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

impl_power_fractal!(BurningShip, "Burning Ship");

// ============================================================================
// Tricorn (Mandelbar)
// ============================================================================

/// The Tricorn fractal (also known as Mandelbar).
///
/// Uses complex conjugation in the iteration:
/// z_{n+1} = conj(z_n)^power + c
///
/// The conjugation creates the characteristic three-pointed shape.
pub struct Tricorn {
    pub power: f64,
}

impl Default for Tricorn {
    fn default() -> Self {
        Tricorn { power: 2.0 }
    }
}

impl Tricorn {
    fn compute_point(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return i;
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                // Conjugation for power=2: conj(z)^2 = (a - bi)^2 = (a^2 - b^2) - 2abi
                let new_re = r2 - i2 + c_re;
                let new_im = -2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                // Conjugation: flip the sign of the imaginary component
                let angle = -power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }
        }

        max_iter
    }

    fn compute_point_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;
        let mut orbit_data = OrbitData::new();

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return FractalResult::escaped(i, Complex64::new(z_re, z_im), orbit_data);
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = r2 - i2 + c_re;
                let new_im = -2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = -power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

impl_power_fractal!(Tricorn, "Tricorn");

// ============================================================================
// Celtic
// ============================================================================

/// The Celtic fractal.
///
/// A variant of the Mandelbrot set where the real part of z^2 is
/// replaced by its absolute value:
///   new_re = |Re(z^2)| + c_re = |a^2 - b^2| + c_re
///   new_im = Im(z^2) + c_im  = 2*a*b + c_im
///
/// Creates intricate Celtic knot-like patterns.
pub struct Celtic {
    pub power: f64,
}

impl Default for Celtic {
    fn default() -> Self {
        Celtic { power: 2.0 }
    }
}

impl Celtic {
    fn compute_point(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return i;
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                // Celtic for power=2: standard z^2+c with abs on real part
                // Must compute both from OLD z_re/z_im before assigning
                let new_re = (r2 - i2 + c_re).abs();
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                // General power via De Moivre, abs on real component
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = (radius * angle.cos() + c_re).abs();
                z_im = radius * angle.sin() + c_im;
            }
        }

        max_iter
    }

    fn compute_point_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;
        let mut orbit_data = OrbitData::new();

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return FractalResult::escaped(i, Complex64::new(z_re, z_im), orbit_data);
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = (r2 - i2 + c_re).abs();
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = (radius * angle.cos() + c_re).abs();
                z_im = radius * angle.sin() + c_im;
            }

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

impl_power_fractal!(Celtic, "Celtic");

// ============================================================================
// Newton's Method Fractal
// ============================================================================

/// Newton's method fractal for z^3 - 1 = 0.
///
/// Uses Newton's root-finding method to visualize which root each point
/// converges to when applying Newton's method to f(z) = z^3 - 1.
///
/// The roots are the cube roots of unity:
/// - root1: z = 1
/// - root2: z = e^(2πi/3) = -0.5 + 0.866i
/// - root3: z = e^(4πi/3) = -0.5 - 0.866i
///
/// Newton's iteration: z_{n+1} = z_n - f(z_n)/f'(z_n)
/// For f(z) = z^3 - 1: z_{n+1} = z - (z^3 - 1) / (3z^2)
pub struct Newton {
    pub tolerance: f64,
}

impl Default for Newton {
    fn default() -> Self {
        Newton { tolerance: 0.001 }
    }
}

impl Fractal for Newton {
    fn name(&self) -> &str {
        "Newton"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![Parameter {
            name: "tolerance".to_string(),
            value: self.tolerance,
            min: 0.0001,
            max: 0.1,
        }]
    }

    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "tolerance" {
            self.tolerance = value.clamp(0.0001, 0.1);
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "tolerance" => Some(self.tolerance),
            _ => None,
        }
    }

    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re = cx;
        let mut z_im = cy;
        let tolerance = self.tolerance;
        let tolerance2 = tolerance * tolerance;

        // Pre-computed roots of z^3 - 1 = 0 (cube roots of unity)
        let root1_re = 1.0;
        let root1_im = 0.0;
        let root2_re = -0.5;
        let root2_im = 0.8660254037844386; // sqrt(3)/2
        let root3_re = -0.5;
        let root3_im = -0.8660254037844386; // -sqrt(3)/2

        for i in 0..max_iter {
            // Check convergence to any root using squared distance (faster)
            let dist2_1 = (z_re - root1_re).powi(2) + (z_im - root1_im).powi(2);
            let dist2_2 = (z_re - root2_re).powi(2) + (z_im - root2_im).powi(2);
            let dist2_3 = (z_re - root3_re).powi(2) + (z_im - root3_im).powi(2);

            if dist2_1 < tolerance2 || dist2_2 < tolerance2 || dist2_3 < tolerance2 {
                return max_iter - i; // Converged - return high iteration count
            }

            // Compute z^2 and z^3 for Newton's method
            let z_re2 = z_re * z_re;
            let z_im2 = z_im * z_im;
            let z_re3 = z_re2 * z_re - 3.0 * z_re * z_im2;
            let z_im3 = 3.0 * z_re2 * z_im - z_im2 * z_im;

            let f_real = z_re3 - 1.0;
            let f_imag = z_im3;

            let deriv_real = 3.0 * (z_re2 - z_im2);
            let deriv_imag = 6.0 * z_re * z_im;

            let denom = deriv_real * deriv_real + deriv_imag * deriv_imag;
            if denom.abs() < 1e-20 {
                break;
            }

            let new_re = z_re - (f_real * deriv_real + f_imag * deriv_imag) / denom;
            let new_im = z_im - (f_imag * deriv_real - f_real * deriv_imag) / denom;

            z_re = new_re;
            z_im = new_im;
        }

        max_iter // Did not converge to a root
    }

    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re = cx;
        let mut z_im = cy;
        let tolerance = self.tolerance;
        let tolerance2 = tolerance * tolerance;
        let mut orbit_data = OrbitData::new();

        let root1_re = 1.0;
        let root1_im = 0.0;
        let root2_re = -0.5;
        let root2_im = 0.8660254037844386;
        let root3_re = -0.5;
        let root3_im = -0.8660254037844386;

        for i in 0..max_iter {
            let dist2_1 = (z_re - root1_re).powi(2) + (z_im - root1_im).powi(2);
            let dist2_2 = (z_re - root2_re).powi(2) + (z_im - root2_im).powi(2);
            let dist2_3 = (z_re - root3_re).powi(2) + (z_im - root3_im).powi(2);

            if dist2_1 < tolerance2 || dist2_2 < tolerance2 || dist2_3 < tolerance2 {
                // Converged - treat as "escaped" for coloring purposes with inverted count
                return FractalResult::escaped(
                    max_iter - i,
                    Complex64::new(z_re, z_im),
                    orbit_data,
                );
            }

            let z_re2 = z_re * z_re;
            let z_im2 = z_im * z_im;
            let z_re3 = z_re2 * z_re - 3.0 * z_re * z_im2;
            let z_im3 = 3.0 * z_re2 * z_im - z_im2 * z_im;

            let f_real = z_re3 - 1.0;
            let f_imag = z_im3;
            let deriv_real = 3.0 * (z_re2 - z_im2);
            let deriv_imag = 6.0 * z_re * z_im;

            let denom = deriv_real * deriv_real + deriv_imag * deriv_imag;
            if denom.abs() < 1e-20 {
                break;
            }

            z_re -= (f_real * deriv_real + f_imag * deriv_imag) / denom;
            z_im -= (f_imag * deriv_real - f_real * deriv_imag) / denom;

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

// ============================================================================
// Biomorph
// ============================================================================

/// Biomorph fractal (Clifford Pickover).
///
/// Uses the standard Mandelbrot-type iteration z = z^power + c, but with
/// a different escape test: a point is considered "inside" if EITHER
///   |Re(z)| < escape_radius  OR  |Im(z)| < escape_radius
/// after the final iteration.
///
/// This produces organic, biological-looking structures (amoebas,
/// microorganisms) hence the name "biomorph."
pub struct Biomorph {
    pub power: f64,
    pub escape_radius: f64,
}

impl Default for Biomorph {
    fn default() -> Self {
        Biomorph {
            power: 3.0,
            escape_radius: 10.0,
        }
    }
}

impl Fractal for Biomorph {
    fn name(&self) -> &str {
        "Biomorph"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "power".to_string(),
                value: self.power,
                min: 2.0,
                max: 8.0,
            },
            Parameter {
                name: "escape_radius".to_string(),
                value: self.escape_radius,
                min: 2.0,
                max: 100.0,
            },
        ]
    }

    fn set_parameter(&mut self, name: &str, value: f64) {
        match name {
            "power" => self.power = value.clamp(2.0, 8.0),
            "escape_radius" => self.escape_radius = value.clamp(2.0, 100.0),
            _ => {}
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "power" => Some(self.power),
            "escape_radius" => Some(self.escape_radius),
            _ => None,
        }
    }

    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re = 0.0_f64;
        let mut z_im = 0.0_f64;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;
        let big_r = self.escape_radius;
        let big_r2 = big_r * big_r;

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            // Standard bailout to prevent overflow
            if r2 + i2 > big_r2 {
                // Pickover biomorph test: if either component is still small, it's "inside"
                if z_re.abs() < big_r || z_im.abs() < big_r {
                    return max_iter - i; // Biomorph region -- high iteration count
                }
                return i; // Escaped
            }

            // z = z^power + c
            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = r2 - i2 + c_re;
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }
        }

        // After all iterations: apply biomorph test on final z
        if z_re.abs() < big_r || z_im.abs() < big_r {
            max_iter // Inside biomorph
        } else {
            0 // Outside
        }
    }

    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re = 0.0_f64;
        let mut z_im = 0.0_f64;
        let c_re = cx;
        let c_im = cy;
        let power = self.power;
        let big_r = self.escape_radius;
        let big_r2 = big_r * big_r;
        let mut orbit_data = OrbitData::new();

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > big_r2 {
                if z_re.abs() < big_r || z_im.abs() < big_r {
                    return FractalResult::escaped(
                        max_iter - i,
                        Complex64::new(z_re, z_im),
                        orbit_data,
                    );
                }
                return FractalResult::escaped(i, Complex64::new(z_re, z_im), orbit_data);
            }

            if (power - 2.0).abs() < POWER2_EPSILON {
                let new_re = r2 - i2 + c_re;
                let new_im = 2.0 * z_re * z_im + c_im;
                z_re = new_re;
                z_im = new_im;
            } else {
                let angle = power * z_im.atan2(z_re);
                let radius = (r2 + i2).powf(power / 2.0);
                z_re = radius * angle.cos() + c_re;
                z_im = radius * angle.sin() + c_im;
            }

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        if z_re.abs() < big_r || z_im.abs() < big_r {
            FractalResult::inside_set(max_iter)
        } else {
            FractalResult::escaped(0, Complex64::new(z_re, z_im), orbit_data)
        }
    }
}

// ============================================================================
// Phoenix Fractal
// ============================================================================

/// Phoenix fractal (Shigehiro Ushiki, 1988).
///
/// A Julia-type iteration with a memory term feeding back the previous z:
///   z_{n+1} = z_n^2 + c + p * z_{n-1}
/// where c is a fixed complex constant and p is the "memory" coefficient.
///
/// The classic Ushiki Phoenix uses c = 0.5667, p = -0.5 which produces
/// the iconic phoenix-shaped connected Julia set.
pub struct Phoenix {
    pub c_real: f64,
    pub c_imag: f64,
    pub memory: f64,
}

impl Default for Phoenix {
    fn default() -> Self {
        Phoenix {
            c_real: 0.5667,
            c_imag: 0.0,
            memory: -0.5,
        }
    }
}

impl Fractal for Phoenix {
    fn name(&self) -> &str {
        "Phoenix"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "c_real".to_string(),
                value: self.c_real,
                min: -2.0,
                max: 2.0,
            },
            Parameter {
                name: "c_imag".to_string(),
                value: self.c_imag,
                min: -2.0,
                max: 2.0,
            },
            Parameter {
                name: "memory".to_string(),
                value: self.memory,
                min: -1.0,
                max: 1.0,
            },
        ]
    }

    fn set_parameter(&mut self, name: &str, value: f64) {
        match name {
            "c_real" => self.c_real = value.clamp(-2.0, 2.0),
            "c_imag" => self.c_imag = value.clamp(-2.0, 2.0),
            "memory" => self.memory = value.clamp(-1.0, 1.0),
            _ => {}
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "c_real" => Some(self.c_real),
            "c_imag" => Some(self.c_imag),
            "memory" => Some(self.memory),
            _ => None,
        }
    }

    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re = cx;
        let mut z_im = cy;
        let mut z_prev_re = 0.0;
        let mut z_prev_im = 0.0;
        let c_re = self.c_real;
        let c_im = self.c_imag;
        let p = self.memory;

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return i;
            }

            // z^2 + c + p * z_prev
            let new_re = r2 - i2 + c_re + p * z_prev_re;
            let new_im = 2.0 * z_re * z_im + c_im + p * z_prev_im;

            z_prev_re = z_re;
            z_prev_im = z_im;
            z_re = new_re;
            z_im = new_im;
        }

        max_iter
    }

    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re = cx;
        let mut z_im = cy;
        let mut z_prev_re = 0.0;
        let mut z_prev_im = 0.0;
        let c_re = self.c_real;
        let c_im = self.c_imag;
        let p = self.memory;
        let mut orbit_data = OrbitData::new();

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return FractalResult::escaped(i, Complex64::new(z_re, z_im), orbit_data);
            }

            let new_re = r2 - i2 + c_re + p * z_prev_re;
            let new_im = 2.0 * z_re * z_im + c_im + p * z_prev_im;

            z_prev_re = z_re;
            z_prev_im = z_im;
            z_re = new_re;
            z_im = new_im;

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

// ============================================================================
// Multibrot Fractal
// ============================================================================

/// Multibrot fractal - Mandelbrot generalized to arbitrary power.
///
/// z_{n+1} = z_n^power + c
///
/// Power 2 gives standard Mandelbrot, power 3 gives the cubic Multibrot
/// (3 lobes), power 4 gives 4 lobes, and so on. Each integer power d
/// produces a set with (d-1) lobes around the origin.
///
/// Delegates to Mandelbrot's compute methods (identical formula, different
/// default power and parameter range).
pub struct Multibrot {
    inner: Mandelbrot,
}

impl Default for Multibrot {
    fn default() -> Self {
        Multibrot {
            inner: Mandelbrot { power: 3.0 },
        }
    }
}

impl Fractal for Multibrot {
    fn name(&self) -> &str {
        "Multibrot"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![Parameter {
            name: "power".to_string(),
            value: self.inner.power,
            min: 2.0,
            max: 10.0,
        }]
    }

    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "power" {
            self.inner.power = value.clamp(2.0, 10.0);
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "power" => Some(self.inner.power),
            _ => None,
        }
    }

    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        self.inner.compute_point(cx, cy, max_iter)
    }

    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        self.inner.compute_point_full(cx, cy, max_iter)
    }
}

// ============================================================================
// Spider Fractal
// ============================================================================

/// The Spider fractal.
///
/// A Mandelbrot variant where the c parameter also evolves each iteration:
///   z_{n+1} = z_n^2 + c_n
///   c_{n+1} = c_n / 2 + z_{n+1}
/// with z_0 = 0, c_0 = pixel coordinate.
///
/// The evolving c creates distinctive spiderweb-like filaments radiating
/// from the main body of the set.
pub struct Spider;

impl Default for Spider {
    fn default() -> Self {
        Spider
    }
}

impl Fractal for Spider {
    fn name(&self) -> &str {
        "Spider"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![]
    }

    fn set_parameter(&mut self, _name: &str, _value: f64) {}

    fn get_parameter(&self, _name: &str) -> Option<f64> {
        None
    }

    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let mut c_re = cx;
        let mut c_im = cy;

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return i;
            }

            // z = z^2 + c
            let new_z_re = r2 - i2 + c_re;
            let new_z_im = 2.0 * z_re * z_im + c_im;
            z_re = new_z_re;
            z_im = new_z_im;

            // c = c/2 + z
            c_re = c_re / 2.0 + z_re;
            c_im = c_im / 2.0 + z_im;
        }

        max_iter
    }

    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let mut c_re = cx;
        let mut c_im = cy;
        let mut orbit_data = OrbitData::new();

        for i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                return FractalResult::escaped(i, Complex64::new(z_re, z_im), orbit_data);
            }

            let new_z_re = r2 - i2 + c_re;
            let new_z_im = 2.0 * z_re * z_im + c_im;
            z_re = new_z_re;
            z_im = new_z_im;

            c_re = c_re / 2.0 + z_re;
            c_im = c_im / 2.0 + z_im;

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

// ============================================================================
// Orbit Trap
// ============================================================================

/// Orbit Trap - Mandelbrot variant that tracks minimum distance to a trap.
///
/// Instead of just counting iterations until escape, tracks the minimum
/// distance the orbit comes to a "trap" point (default: origin).
/// Creates beautiful patterns and tendrils around the Mandelbrot set.
pub struct OrbitTrap {
    pub trap_x: f64,
    pub trap_y: f64,
}

impl Default for OrbitTrap {
    fn default() -> Self {
        OrbitTrap {
            trap_x: 0.0,
            trap_y: 0.0,
        }
    }
}

impl Fractal for OrbitTrap {
    fn name(&self) -> &str {
        "Orbit Trap"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "trap_x".to_string(),
                value: self.trap_x,
                min: -2.0,
                max: 2.0,
            },
            Parameter {
                name: "trap_y".to_string(),
                value: self.trap_y,
                min: -2.0,
                max: 2.0,
            },
        ]
    }

    fn set_parameter(&mut self, name: &str, value: f64) {
        match name {
            "trap_x" => self.trap_x = value.clamp(-2.0, 2.0),
            "trap_y" => self.trap_y = value.clamp(-2.0, 2.0),
            _ => {}
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "trap_x" => Some(self.trap_x),
            "trap_y" => Some(self.trap_y),
            _ => None,
        }
    }

    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;

        let mut min_distance_sq = f64::MAX;

        for _i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                let dist = min_distance_sq.sqrt();
                let trap_value = (1.0 / (1.0 + dist * 10.0) * max_iter as f64) as u32;
                return trap_value.min(max_iter - 1);
            }

            let dx = z_re - self.trap_x;
            let dy = z_im - self.trap_y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < min_distance_sq {
                min_distance_sq = dist_sq;
            }

            let new_re = r2 - i2 + c_re;
            let new_im = 2.0 * z_re * z_im + c_im;
            z_re = new_re;
            z_im = new_im;
        }

        max_iter
    }

    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let mut z_re: f64 = 0.0;
        let mut z_im: f64 = 0.0;
        let c_re = cx;
        let c_im = cy;
        let mut orbit_data = OrbitData::new();
        let mut min_distance_sq = f64::MAX;

        for _i in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                let dist = min_distance_sq.sqrt();
                let trap_value = (1.0 / (1.0 + dist * 10.0) * max_iter as f64) as u32;
                let iters = trap_value.min(max_iter - 1);
                return FractalResult::escaped(iters, Complex64::new(z_re, z_im), orbit_data);
            }

            let dx = z_re - self.trap_x;
            let dy = z_im - self.trap_y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < min_distance_sq {
                min_distance_sq = dist_sq;
            }

            let new_re = r2 - i2 + c_re;
            let new_im = 2.0 * z_re * z_im + c_im;
            z_re = new_re;
            z_im = new_im;

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

// ============================================================================
// Pickover Stalk
// ============================================================================

/// Pickover Stalk - Mandelbrot variant named after Clifford Pickover.
///
/// Tracks how close the orbit comes to the real and imaginary axes,
/// creating "stalk-like" structures that extend from the set.
/// Creates beautiful organic patterns resembling plants/stalks.
pub struct PickoverStalk {
    pub stalk_thickness: f64,
    pub stalk_intensity: f64,
}

impl Default for PickoverStalk {
    fn default() -> Self {
        PickoverStalk {
            stalk_thickness: 0.1,
            stalk_intensity: 20.0,
        }
    }
}

impl Fractal for PickoverStalk {
    fn name(&self) -> &str {
        "Pickover Stalk"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "thickness".to_string(),
                value: self.stalk_thickness,
                min: 0.01,
                max: 1.0,
            },
            Parameter {
                name: "intensity".to_string(),
                value: self.stalk_intensity,
                min: 1.0,
                max: 100.0,
            },
        ]
    }

    fn set_parameter(&mut self, name: &str, value: f64) {
        match name {
            "thickness" => self.stalk_thickness = value.clamp(0.01, 1.0),
            "intensity" => self.stalk_intensity = value.clamp(1.0, 100.0),
            _ => {}
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "thickness" => Some(self.stalk_thickness),
            "intensity" => Some(self.stalk_intensity),
            _ => None,
        }
    }

    fn compute(&self, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let c_re = cx;
        let c_im = cy;
        let mut z_re = cx;
        let mut z_im = cy;

        let mut trap_distance = f64::MAX;

        for iteration in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                let normalized = trap_distance / self.stalk_thickness;
                let stalk_brightness = 1.0 / (1.0 + normalized * self.stalk_intensity);
                let final_value = (stalk_brightness * iteration as f64) as u32;
                return final_value.max(1);
            }

            let dist_to_real = z_im.abs();
            let dist_to_imag = z_re.abs();
            let smallest_distance = dist_to_real.min(dist_to_imag);

            if smallest_distance < trap_distance {
                trap_distance = smallest_distance;
            }

            let new_re = r2 - i2 + c_re;
            let new_im = 2.0 * z_re * z_im + c_im;
            z_re = new_re;
            z_im = new_im;
        }

        max_iter
    }

    fn compute_full(&self, cx: f64, cy: f64, max_iter: u32) -> FractalResult {
        let c_re = cx;
        let c_im = cy;
        let mut z_re = cx;
        let mut z_im = cy;
        let mut orbit_data = OrbitData::new();
        let mut trap_distance = f64::MAX;

        for iteration in 0..max_iter {
            let r2 = z_re * z_re;
            let i2 = z_im * z_im;

            if r2 + i2 > BAILOUT_R2 {
                let normalized = trap_distance / self.stalk_thickness;
                let stalk_brightness = 1.0 / (1.0 + normalized * self.stalk_intensity);
                let final_value = (stalk_brightness * iteration as f64) as u32;
                let iters = final_value.max(1);
                return FractalResult::escaped(iters, Complex64::new(z_re, z_im), orbit_data);
            }

            let dist_to_real = z_im.abs();
            let dist_to_imag = z_re.abs();
            let smallest_distance = dist_to_real.min(dist_to_imag);

            if smallest_distance < trap_distance {
                trap_distance = smallest_distance;
            }

            let new_re = r2 - i2 + c_re;
            let new_im = 2.0 * z_re * z_im + c_im;
            z_re = new_re;
            z_im = new_im;

            orbit_data.update(Complex64::new(z_re, z_im));
        }

        FractalResult::inside_set(max_iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandelbrot_outside() {
        let m = Mandelbrot::default();
        let result = m.compute(2.0, 0.0, 100);
        assert!(result < 100, "Point outside Mandelbrot should escape");
    }

    #[test]
    fn test_mandelbrot_cardioid() {
        let m = Mandelbrot::default();
        let result = m.compute(0.25, 0.0, 100);
        assert!(result >= 100, "Cardioid point should NOT escape");
    }

    #[test]
    fn test_julia_center() {
        let j = Julia::default();
        let result = j.compute(0.0, 0.0, 100);
        assert!(
            result >= 50,
            "Origin should take many iterations for default Julia"
        );
    }

    #[test]
    fn test_julia_outside() {
        let j = Julia::default();
        let result = j.compute(1.0, 0.0, 100);
        assert!(result < 100, "Point outside Julia should escape");
    }

    #[test]
    fn test_burning_ship_center() {
        let b = BurningShip::default();
        let result = b.compute(-0.5, -0.5, 100);
        assert!(
            result >= 50,
            "Center of Burning Ship should take many iterations"
        );
    }

    #[test]
    fn test_burning_ship_outside() {
        let b = BurningShip::default();
        let result = b.compute(2.0, 2.0, 100);
        assert!(result < 10, "Far outside should escape quickly");
    }

    #[test]
    fn test_pickover_stalk_parameters() {
        let mut ps = PickoverStalk::default();

        // Test default values (updated defaults)
        assert!(
            (ps.stalk_thickness - 0.1).abs() < 0.001,
            "Default thickness should be 0.1"
        );
        assert!(
            (ps.stalk_intensity - 20.0).abs() < 0.001,
            "Default intensity should be 20.0"
        );

        // Test set_parameter
        ps.set_parameter("thickness", 0.5);
        assert!(
            (ps.stalk_thickness - 0.5).abs() < 0.001,
            "Thickness should be set to 0.5"
        );

        ps.set_parameter("intensity", 50.0);
        assert!(
            (ps.stalk_intensity - 50.0).abs() < 0.001,
            "Intensity should be set to 50.0"
        );

        // Test get_parameter
        assert!((ps.get_parameter("thickness").unwrap() - 0.5).abs() < 0.001);
        assert!((ps.get_parameter("intensity").unwrap() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_pickover_stalk_computation() {
        let ps = PickoverStalk::default();

        // Test a point that should escape and produce a stalk value
        let result = ps.compute(0.5, 0.5, 100);

        // Should return a value based on stalk proximity, not max_iter
        assert!(
            result < 100,
            "Point outside set should escape with stalk value"
        );

        // Test center point (should be in set)
        let result_center = ps.compute(-0.5, 0.0, 100);
        assert_eq!(result_center, 100, "Center should be in set");
    }

    #[test]
    fn test_pickover_stalk_parameter_effect() {
        let mut ps = PickoverStalk::default();

        ps.set_parameter("thickness", 0.01);
        let with_thin = ps.compute(0.8, 0.2, 100);

        ps.set_parameter("thickness", 1.0);
        let with_thick = ps.compute(0.8, 0.2, 100);

        assert!(
            with_thin < 100 && with_thick < 100,
            "Both should escape: thin={}, thick={}",
            with_thin,
            with_thick
        );
    }

    // ========================================================================
    // Tests for previously untested fractals
    // ========================================================================

    #[test]
    fn test_tricorn_center() {
        let t = Tricorn::default();
        // Origin should be in the Tricorn set
        let result = t.compute(0.0, 0.0, 200);
        assert_eq!(result, 200, "Origin should be in Tricorn set");
    }

    #[test]
    fn test_tricorn_outside() {
        let t = Tricorn::default();
        let result = t.compute(2.0, 2.0, 100);
        assert!(result < 10, "Far outside should escape quickly");
    }

    #[test]
    fn test_celtic_center() {
        let c = Celtic::default();
        let result = c.compute(0.0, 0.0, 200);
        // Origin should be in the set (z=0, z^2+c=c=0, stays at 0)
        assert_eq!(result, 200, "Origin should be in Celtic set");
    }

    #[test]
    fn test_celtic_outside() {
        let c = Celtic::default();
        let result = c.compute(2.0, 2.0, 100);
        assert!(result < 10, "Far outside should escape quickly");
    }

    #[test]
    fn test_newton_convergence() {
        let n = Newton::default();
        // Point near root z=1 should converge quickly (high return value = early convergence)
        let result = n.compute(0.9, 0.0, 100);
        assert!(
            result > 90,
            "Near root1 should converge quickly, got {}",
            result
        );
    }

    #[test]
    fn test_newton_far_from_roots() {
        let n = Newton::default();
        // Point at origin: z=0 is a singularity for Newton's method on z^3-1
        // f'(0) = 0, so this should hit the singularity guard
        let result = n.compute(0.0, 0.0, 100);
        assert_eq!(result, 100, "Origin should hit singularity guard");
    }

    #[test]
    fn test_newton_all_three_roots() {
        let n = Newton::default();
        // Near root1 (1, 0)
        let r1 = n.compute(1.01, 0.0, 100);
        assert!(r1 > 90, "Near root1 should converge, got {}", r1);

        // Near root2 (-0.5, sqrt(3)/2 ≈ 0.866)
        let r2 = n.compute(-0.49, 0.87, 100);
        assert!(r2 > 90, "Near root2 should converge, got {}", r2);

        // Near root3 (-0.5, -sqrt(3)/2 ≈ -0.866)
        let r3 = n.compute(-0.49, -0.87, 100);
        assert!(r3 > 90, "Near root3 should converge, got {}", r3);
    }

    #[test]
    fn test_biomorph_convergence() {
        let b = Biomorph::default();
        // Near root z=1 should converge (return max_iter - i, high value)
        let result = b.compute(0.9, 0.0, 100);
        assert!(result > 80, "Near root should converge, got {}", result);
    }

    #[test]
    fn test_biomorph_escape() {
        let b = Biomorph::default();
        // Biomorph uses Newton iteration which can converge near roots (returning high values)
        // or escape beyond escape_radius. Very far points should escape.
        let result = b.compute(10.0, 10.0, 100);
        // The result should be a low escape iteration or a convergence value
        assert!(
            result < 100,
            "Far outside should not reach max_iter, got {}",
            result
        );
    }

    #[test]
    fn test_phoenix_outside() {
        let p = Phoenix::default();
        let result = p.compute(2.0, 2.0, 100);
        assert!(result < 20, "Far outside should escape quickly");
    }

    #[test]
    fn test_phoenix_near_center() {
        let p = Phoenix::default();
        // With Ushiki defaults (c=0.5667, p=-0.5) the origin escapes relatively
        // quickly but should survive more than 1 iteration
        let result = p.compute(0.0, 0.0, 200);
        assert!(
            result >= 2 && result < 200,
            "Origin should escape after a few iterations, got {}",
            result
        );
    }

    #[test]
    fn test_multibrot_center() {
        let m = Multibrot::default(); // power=3
                                      // Origin should be in set: z=0 -> z^3+0=0 forever
        let result = m.compute(0.0, 0.0, 100);
        assert_eq!(result, 100, "Origin should be in Multibrot set");
    }

    #[test]
    fn test_multibrot_outside() {
        let m = Multibrot::default();
        let result = m.compute(2.0, 0.0, 100);
        assert!(result < 10, "Far outside should escape quickly");
    }

    #[test]
    fn test_spider_center() {
        let s = Spider::default();
        // z=0, c=0: iteration stays at 0 forever (alternating +0/-0)
        let result = s.compute(0.0, 0.0, 100);
        assert_eq!(result, 100, "Origin should be in Spider set");
    }

    #[test]
    fn test_spider_outside() {
        let s = Spider::default();
        let result = s.compute(2.0, 2.0, 100);
        assert!(result < 10, "Far outside should escape quickly");
    }

    #[test]
    fn test_orbit_trap_outside() {
        let ot = OrbitTrap::default();
        // Point that escapes should return a trap-based value < max_iter
        let result = ot.compute(0.5, 0.5, 200);
        assert!(result < 200, "Outside point should escape with trap value");
    }

    #[test]
    fn test_orbit_trap_in_set() {
        let ot = OrbitTrap::default();
        // Origin: z=0 -> 0^2+0 = 0, stays forever
        let result = ot.compute(0.0, 0.0, 100);
        assert_eq!(result, 100, "Origin should be in set");
    }

    // ========================================================================
    // compute_full() tests - orbit data and final_z
    // ========================================================================

    #[test]
    fn test_mandelbrot_compute_full_escaped() {
        let m = Mandelbrot::default();
        let result = m.compute_full(2.0, 0.0, 100);
        assert!(result.escaped, "Point (2,0) should escape");
        assert!(result.final_z.is_some(), "Should have final_z");
        let z = result.final_z.unwrap();
        assert!(
            z.norm_sqr() > 4.0,
            "final_z should be outside bailout radius"
        );
    }

    #[test]
    fn test_mandelbrot_compute_full_inside() {
        let m = Mandelbrot::default();
        let result = m.compute_full(0.0, 0.0, 100);
        assert!(!result.escaped, "Origin should be inside set");
        assert!(
            result.final_z.is_none(),
            "Inside-set should have no final_z"
        );
    }

    #[test]
    fn test_compute_full_orbit_data_populated() {
        let m = Mandelbrot::default();
        // Use a point clearly outside the set that escapes quickly
        let result = m.compute_full(0.5, 0.5, 200);
        assert!(result.escaped, "Point (0.5, 0.5) should escape");
        let od = result.orbit_data;
        // After at least one iteration, min distances should not be infinity
        assert!(
            od.min_distance_to_origin < f64::INFINITY,
            "Orbit data should track distance to origin"
        );
        assert!(
            od.min_distance_to_real_axis < f64::INFINITY,
            "Orbit data should track distance to real axis"
        );
        assert!(
            od.min_distance_to_imag_axis < f64::INFINITY,
            "Orbit data should track distance to imag axis"
        );
    }

    #[test]
    fn test_julia_compute_full_orbit_data() {
        let j = Julia::default();
        let result = j.compute_full(1.0, 0.0, 100);
        assert!(result.escaped);
        assert!(result.final_z.is_some());
        assert!(result.orbit_data.min_distance_to_origin < f64::INFINITY);
    }

    #[test]
    fn test_newton_compute_full_convergence() {
        let n = Newton::default();
        // Near root1, should converge and return as escaped with high iteration count
        let result = n.compute_full(0.9, 0.0, 100);
        assert!(result.escaped, "Convergence should be treated as escaped");
        assert!(
            result.iterations > 80,
            "Should have high iter count from convergence"
        );
        assert!(result.final_z.is_some());
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_mandelbrot_max_iter_1() {
        let m = Mandelbrot::default();
        // With max_iter=1, point at origin: z=0, i=0: r2+i2=0 < 4, compute z^2+c,
        // loop ends -> return max_iter=1
        let result = m.compute(0.0, 0.0, 1);
        assert_eq!(result, 1, "Single iteration, origin stays -> max_iter");

        // Far point: z starts at 0, so at i=0 the escape check sees |z|^2=0 < 4.
        // After one iteration z=3+0i, but the loop ends -> return max_iter=1.
        // It only escapes on the NEXT iteration's check (i=1), which doesn't run.
        let result = m.compute(3.0, 0.0, 1);
        assert_eq!(
            result, 1,
            "z starts at 0, so first escape check passes even for far c"
        );

        // With max_iter=2, the far point should escape at i=1
        let result = m.compute(3.0, 0.0, 2);
        assert_eq!(result, 1, "Far point should escape at i=1 (z=3, |z|^2=9>4)");
    }

    #[test]
    fn test_parameter_clamping() {
        let mut m = Mandelbrot::default();
        m.set_parameter("power", 100.0);
        assert!(
            (m.get_parameter("power").unwrap() - 8.0).abs() < 0.001,
            "Power should be clamped to 8.0"
        );

        m.set_parameter("power", -5.0);
        assert!(
            (m.get_parameter("power").unwrap() - 1.0).abs() < 0.001,
            "Power should be clamped to 1.0"
        );
    }

    #[test]
    fn test_julia_parameter_clamping() {
        let mut j = Julia::default();
        j.set_parameter("c_real", 10.0);
        assert!(
            (j.get_parameter("c_real").unwrap() - 2.0).abs() < 0.001,
            "c_real should be clamped to 2.0"
        );

        j.set_parameter("c_imag", -10.0);
        assert!(
            (j.get_parameter("c_imag").unwrap() - (-2.0)).abs() < 0.001,
            "c_imag should be clamped to -2.0"
        );
    }

    #[test]
    fn test_unknown_parameter_ignored() {
        let mut m = Mandelbrot::default();
        let before = m.get_parameter("power").unwrap();
        m.set_parameter("nonexistent", 42.0);
        let after = m.get_parameter("power").unwrap();
        assert_eq!(
            before, after,
            "Unknown parameter should not change anything"
        );
        assert_eq!(m.get_parameter("nonexistent"), None);
    }

    #[test]
    fn test_compute_consistency() {
        // compute() and compute_full() should return the same iteration count
        let m = Mandelbrot::default();
        for &(cx, cy) in &[(0.0, 0.0), (0.3, 0.5), (2.0, 0.0), (-0.5, 0.0), (0.25, 0.0)] {
            let simple = m.compute(cx, cy, 200);
            let full = m.compute_full(cx, cy, 200);
            assert_eq!(
                simple, full.iterations,
                "compute() and compute_full() should match for ({}, {}): {} vs {}",
                cx, cy, simple, full.iterations
            );
        }
    }

    #[test]
    fn test_compute_full_consistency_julia() {
        let j = Julia::default();
        for &(cx, cy) in &[(0.0, 0.0), (1.0, 0.0), (0.5, 0.5), (-1.0, -1.0)] {
            let simple = j.compute(cx, cy, 200);
            let full = j.compute_full(cx, cy, 200);
            assert_eq!(
                simple, full.iterations,
                "Julia compute/compute_full mismatch at ({}, {}): {} vs {}",
                cx, cy, simple, full.iterations
            );
        }
    }
}
