use crate::fractal::{Fractal, FractalType, Parameter};
use std::collections::HashMap;

/// Metadata about a fractal type
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FractalMetadata {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub default_center: (f64, f64),
    pub default_zoom: f64,
    pub category: FractalCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalCategory {
    MandelbrotLike,
    JuliaLike,
    Special,
}

/// Factory trait for creating fractal instances
#[allow(dead_code)]
pub trait FractalFactory: Send + Sync {
    fn create(&self) -> Box<dyn Fractal>;
    fn metadata(&self) -> FractalMetadata;
    fn default_parameters(&self) -> Vec<Parameter>;
}

/// Registry of all available fractal types
pub struct FractalRegistry {
    factories: HashMap<FractalType, Box<dyn FractalFactory>>,
}

impl Default for FractalRegistry {
    fn default() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }
}

#[allow(dead_code)]
impl FractalRegistry {
    /// Register all built-in fractal types
    fn register_defaults(&mut self) {
        self.register(FractalType::Mandelbrot, MandelbrotFactory);
        self.register(FractalType::Julia, JuliaFactory);
        self.register(FractalType::BurningShip, BurningShipFactory);
        self.register(FractalType::Tricorn, TricornFactory);
        self.register(FractalType::Celtic, CelticFactory);
        self.register(FractalType::Newton, NewtonFactory);
        self.register(FractalType::Biomorph, BiomorphFactory);
        self.register(FractalType::Phoenix, PhoenixFactory);
        self.register(FractalType::Multibrot, MultibrotFactory);
        self.register(FractalType::Spider, SpiderFactory);
        self.register(FractalType::OrbitTrap, OrbitTrapFactory);
        self.register(FractalType::PickoverStalk, PickoverStalkFactory);
    }

    /// Register a fractal factory
    pub fn register<F: FractalFactory + 'static>(&mut self, fractal_type: FractalType, factory: F) {
        self.factories.insert(fractal_type, Box::new(factory));
    }

    /// Create a fractal instance by type
    pub fn create(&self, fractal_type: FractalType) -> Option<Box<dyn Fractal>> {
        self.factories.get(&fractal_type).map(|f| f.create())
    }

    /// Get metadata for a fractal type
    pub fn metadata(&self, fractal_type: FractalType) -> Option<FractalMetadata> {
        self.factories.get(&fractal_type).map(|f| f.metadata())
    }

    /// Get default parameters for a fractal type
    pub fn default_parameters(&self, fractal_type: FractalType) -> Option<Vec<Parameter>> {
        self.factories
            .get(&fractal_type)
            .map(|f| f.default_parameters())
    }

    /// Get all registered fractal types
    pub fn all_types(&self) -> Vec<FractalType> {
        let mut types: Vec<_> = self.factories.keys().copied().collect();
        types.sort_by_key(|t| format!("{:?}", t)); // Sort alphabetically by name
        types
    }

    /// Check if a fractal type is registered
    pub fn is_registered(&self, fractal_type: FractalType) -> bool {
        self.factories.contains_key(&fractal_type)
    }
}

// Factory implementations for each fractal type
use crate::fractal::*;

struct MandelbrotFactory;
impl FractalFactory for MandelbrotFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Mandelbrot::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "mandelbrot".to_string(),
            display_name: "Mandelbrot".to_string(),
            description: Some("The classic Mandelbrot set".to_string()),
            default_center: (-0.5, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::MandelbrotLike,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Mandelbrot::default().parameters()
    }
}

struct JuliaFactory;
impl FractalFactory for JuliaFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Julia::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "julia".to_string(),
            display_name: "Julia".to_string(),
            description: Some("Julia sets with variable c parameter".to_string()),
            default_center: (0.0, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::JuliaLike,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Julia::default().parameters()
    }
}

struct BurningShipFactory;
impl FractalFactory for BurningShipFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(BurningShip::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "burning_ship".to_string(),
            display_name: "Burning Ship".to_string(),
            description: Some("Burning Ship fractal with absolute values".to_string()),
            default_center: (-0.5, -0.5),
            default_zoom: 1.0,
            category: FractalCategory::MandelbrotLike,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        BurningShip::default().parameters()
    }
}

struct TricornFactory;
impl FractalFactory for TricornFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Tricorn::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "tricorn".to_string(),
            display_name: "Tricorn".to_string(),
            description: Some("Tricorn/Mandelbar fractal".to_string()),
            default_center: (0.0, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::MandelbrotLike,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Tricorn::default().parameters()
    }
}

struct CelticFactory;
impl FractalFactory for CelticFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Celtic::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "celtic".to_string(),
            display_name: "Celtic".to_string(),
            description: Some("Celtic fractal variant".to_string()),
            default_center: (0.0, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::MandelbrotLike,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Celtic::default().parameters()
    }
}

struct NewtonFactory;
impl FractalFactory for NewtonFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Newton::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "newton".to_string(),
            display_name: "Newton".to_string(),
            description: Some("Newton's method fractal for z³ - 1 = 0".to_string()),
            default_center: (0.0, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::Special,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Newton::default().parameters()
    }
}

struct BiomorphFactory;
impl FractalFactory for BiomorphFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Biomorph::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "biomorph".to_string(),
            display_name: "Biomorph".to_string(),
            description: Some("Biomorph fractal with escape conditions".to_string()),
            default_center: (0.0, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::Special,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Biomorph::default().parameters()
    }
}

struct PhoenixFactory;
impl FractalFactory for PhoenixFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Phoenix::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "phoenix".to_string(),
            display_name: "Phoenix".to_string(),
            description: Some("Phoenix fractal with memory term".to_string()),
            default_center: (0.0, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::Special,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Phoenix::default().parameters()
    }
}

struct MultibrotFactory;
impl FractalFactory for MultibrotFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Multibrot::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "multibrot".to_string(),
            display_name: "Multibrot".to_string(),
            description: Some("Generalized Mandelbrot with variable power".to_string()),
            default_center: (0.0, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::MandelbrotLike,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Multibrot::default().parameters()
    }
}

struct SpiderFactory;
impl FractalFactory for SpiderFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(Spider)
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "spider".to_string(),
            display_name: "Spider".to_string(),
            description: Some("Spider fractal with evolving c parameter".to_string()),
            default_center: (0.0, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::Special,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        Spider.parameters()
    }
}

struct OrbitTrapFactory;
impl FractalFactory for OrbitTrapFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(OrbitTrap::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "orbit_trap".to_string(),
            display_name: "Orbit Trap".to_string(),
            description: Some("Mandelbrot with orbit trap coloring".to_string()),
            default_center: (-0.5, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::Special,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        OrbitTrap::default().parameters()
    }
}

struct PickoverStalkFactory;
impl FractalFactory for PickoverStalkFactory {
    fn create(&self) -> Box<dyn Fractal> {
        Box::new(PickoverStalk::default())
    }

    fn metadata(&self) -> FractalMetadata {
        FractalMetadata {
            id: "pickover_stalk".to_string(),
            display_name: "Pickover Stalk".to_string(),
            description: Some("Pickover stalk orbit trap".to_string()),
            default_center: (-0.5, 0.0),
            default_zoom: 1.0,
            category: FractalCategory::Special,
        }
    }

    fn default_parameters(&self) -> Vec<Parameter> {
        PickoverStalk::default().parameters()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_default() {
        let registry = FractalRegistry::default();
        assert_eq!(registry.all_types().len(), 12);
        assert!(registry.is_registered(FractalType::Mandelbrot));
        assert!(registry.is_registered(FractalType::Julia));
    }

    #[test]
    fn test_registry_create() {
        let registry = FractalRegistry::default();
        let fractal = registry.create(FractalType::Mandelbrot);
        assert!(fractal.is_some());
    }

    #[test]
    fn test_registry_metadata() {
        let registry = FractalRegistry::default();
        let metadata = registry.metadata(FractalType::Mandelbrot);
        assert!(metadata.is_some());
        let meta = metadata.unwrap();
        assert_eq!(meta.display_name, "Mandelbrot");
        assert_eq!(meta.default_center, (-0.5, 0.0));
    }

    #[test]
    fn test_registry_parameters() {
        let registry = FractalRegistry::default();
        let params = registry.default_parameters(FractalType::Mandelbrot);
        assert!(params.is_some());
        let params = params.unwrap();
        assert!(!params.is_empty());
    }
}
