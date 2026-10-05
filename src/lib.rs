//! wearable-data-ingestion — the Vital/Junction integration pattern.
//!
//! Pull wearable series (Oura/Apple Health via an aggregation API, or the
//! bundled fixture provider for offline dev), **normalize to FHIR R4
//! Observations** (LOINC-coded), store them, and detect basic trends.
//! The provider is an enum: `Fixture` (committed data) or `Vital` (the
//! real aggregation API when configured) — tests never touch the network.

pub mod normalize;
pub mod provider;
pub mod routes;
pub mod state;
pub mod trends;

pub use state::AppState;
