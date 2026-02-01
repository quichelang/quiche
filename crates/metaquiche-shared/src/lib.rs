//! Shared utilities for the Quiche metaquiche compilers
//!
//! This crate provides common functionality used by both:
//! - metaquiche-host (Rust-based bootstrap compiler)
//! - metaquiche-native (Self-hosted Quiche compiler)

pub mod macros;
pub mod telemetry;
pub mod templates;
