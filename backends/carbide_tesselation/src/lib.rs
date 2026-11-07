//! # Carbide tesselation
//!
//! An abstraction over tesselators, which turn [DrawShape](carbide_core::draw::DrawShape)s
//! into lists of triangles that renderers can draw.
//!
//! Implementation crates (like `carbide_lyon`) implement the [Tesselator] trait and register
//! an instance into the environment using the [TesselatorKey]. Renderers look the tesselator
//! up from the environment, without needing to know which implementation is used.
//!
//! ## Registering an implementation
//! ```ignore
//! use std::any::TypeId;
//! use std::pin::Pin;
//! use carbide_core::environment::Environment;
//! use carbide_core::plugin::Plugin;
//! use carbide_core::register;
//! use carbide_tesselation::TesselatorKey;
//!
//! fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
//!     Box::pin(async {
//!         TesselatorKey::insert(env, MyTesselator::new());
//!     })
//! }
//!
//! register!(Plugin {
//!     dependencies: &[],
//!     registers: TypeId::of::<TesselatorKey>(),
//!     register
//! });
//! ```

mod tesselator;
pub mod triangle;
pub mod vertex;
pub mod stroke_vertex;

pub use tesselator::*;
