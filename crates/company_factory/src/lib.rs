//! Company-hosted development slice.
//!
//! Sign-in is the operator's identity provider. Warp's hosted account service,
//! proprietary factories, and the headless terminal are not this crate.

mod byok;
mod config;
mod http;
mod oidc;
mod repo;
mod session;

pub use config::{Config, ConfigError, OidcConfig};
pub use http::{Server, serve};
