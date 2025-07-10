//! Async process execution and I/O management for the EJ framework.
//!
//! Provides async utilities for spawning, monitoring, and controlling external processes
//! with real-time output capture and timeout handling using tokio.
//!
//! # Usage
//!
//! ```rust
//! use ej_io::runner::{Runner, RunEvent};
//! use std::sync::{Arc, atomic::AtomicBool};
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a process runner
//!     let runner = Runner::new("echo", vec!["Hello, World!"]);
//!     let (tx, mut rx) = mpsc::channel(100);
//!     let should_stop = Arc::new(AtomicBool::new(false));
//!
//!     // Run the process with event handling
//!     let exit_status = runner.run(tx, should_stop).await;
//!
//!     // Handle events asynchronously
//!     while let Some(event) = rx.recv().await {
//!         match event {
//!             RunEvent::ProcessNewOutputLine(line) => println!("Output: {}", line),
//!             RunEvent::ProcessEnd(success) => println!("Process ended: {}", success),
//!             _ => {}
//!         }
//!     }
//! }
//! ```

pub mod process;
pub mod runner;
