//! Shared utilities for the Quiche metaquiche compilers
//!
//! This crate provides common functionality used by both:
//! - metaquiche-host (Rust-based bootstrap compiler)
//! - metaquiche-native (Self-hosted Quiche compiler)

// Initialize i18n at crate root (required by rust-i18n)
rust_i18n::i18n!("locales", fallback = "en-US");

pub mod error_exit;
pub mod i18n;
pub mod macros;
pub mod telemetry;
pub mod template;
