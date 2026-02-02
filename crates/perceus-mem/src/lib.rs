//! # Perceus-Mem
//!
//! A memory management library with generation-based reference counting and regions.
//!
//! Inspired by Microsoft Research's Perceus algorithm, Vala's GObject ARC,
//! and Lobster's compile-time ownership inference.
//!
//! ## Features
//!
//! - **Generation-validated handles**: Prevents use-after-free with runtime generation checks
//! - **Region-based allocation**: Arena-style bulk alloc/dealloc for scoped data
//! - **FBIP (Functional-But-In-Place)**: In-place mutation when ref_count == 1
//! - **Weak references**: Non-owning references for cycle prevention
//! - **QCell-based**: Compile-time checked interior mutability (no RefCell panics)
//! - **Generic thread-safety**: `SingleThreaded` or `ThreadSafe` via `AtomicPolicy` trait
//!
//! ## Quick Start
//!
//! ```rust
//! use perceus_mem::{Store, Handle};
//!
//! let mut store: Store<String> = Store::new();
//! let handle = store.alloc("hello".to_string());
//! println!("{}", store.get(&handle).unwrap());
//! store.release(handle); // Freed when ref count hits 0
//! ```

mod generation;
mod handle;
mod inline;
mod managed;
mod policy;
mod region;
mod store;
mod weak;

pub use generation::{GenIndex, Generation};
pub use handle::Handle;
pub use inline::Inline;
pub use managed::Managed;
pub use policy::{AtomicPolicy, SingleThreaded, ThreadSafe};
pub use region::{Region, RegionHandle};
pub use store::{GenericStore, Store, ThreadSafeStore};
pub use weak::Weak;
