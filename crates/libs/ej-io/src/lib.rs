//! Process execution and I/O management for the EJ framework.
//!
//! Provides utilities for spawning, monitoring, and controlling external processes
//! with real-time output capture and timeout handling.
//!
//! # Usage
//!
//! ```rust
//! use ej_io::runner::{Runner, RunEvent};
//! use std::sync::{Arc, atomic::AtomicBool, mpsc};
//!
//! // Create a process runner
//! let runner = Runner::new("echo", vec!["Hello, World!"]);
//! let (tx, rx) = mpsc::channel();
//! let should_stop = Arc::new(AtomicBool::new(false));
//!
//! // Run the process with event handling
//! let exit_status = runner.run(tx, should_stop);
//!
//! // Handle events
//! while let Ok(event) = rx.try_recv() {
//!     match event {
//!         RunEvent::ProcessNewOutputLine(line) => println!("Output: {}", line),
//!         RunEvent::ProcessEnd(success) => println!("Process ended: {}", success),
//!         _ => {}
//!     }
//! }
//! ```

pub mod process;
pub mod runner;
