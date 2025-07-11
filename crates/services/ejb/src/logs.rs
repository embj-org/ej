//! Log handling and output management for the EJ Builder Service.
//!
//! Provides functionality for:
//! - Dumping execution logs to temporary files
//! - Stripping ANSI escape codes from log output
//! - Writing logs to various output destinations
//! - Managing log file creation and cleanup

use std::{fs::File, io::Write, path::PathBuf};

use crate::{prelude::*, run_output::EjRunOutput};
use strip_ansi_escapes::strip;
use tracing::{error, info};

/// Dumps execution logs to a temporary file.
///
/// Creates a temporary file and writes all collected logs to it.
/// Useful for debugging and log persistence.
pub fn dump_logs_to_temporary_file(output: &EjRunOutput) -> Result<()> {
    match create_temp_and_dump(output) {
        Ok(path) => {
            info!("Logs written to temporary file: {:?}", path);
        }
        Err(e) => {
            error!("Failed to create temporary file and dump logs: {}", e);
        }
    }
    Ok(())
}

fn strip_ansi_codes(input: &str) -> String {
    String::from_utf8_lossy(&strip(input.as_bytes())).to_string()
}

pub fn create_temp_and_dump(output: &EjRunOutput) -> Result<std::path::PathBuf> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let filename = format!("ej_logs_{}.txt", timestamp);
    let path = PathBuf::from(&filename);
    let mut file = File::create(&path)?;
    dump_logs_internal(output, &mut file, true)?;
    Ok(path)
}

pub fn dump_logs<W: Write>(output: &EjRunOutput, writer: W) -> Result<()> {
    dump_logs_internal(output, writer, false)
}

fn dump_logs_internal<W: Write>(
    output: &EjRunOutput,
    mut writer: W,
    strip_ansi: bool,
) -> Result<()> {
    for board in output.config.boards.iter() {
        for board_config in board.configs.iter() {
            let key = board_config.id;
            if let Some(logs) = output.logs.get(&key) {
                writeln!(writer, "========================")?;
                writeln!(
                    writer,
                    "Log outputs for {} {}",
                    board.name, board_config.name
                )?;
                writeln!(writer, "========================")?;

                if strip_ansi {
                    for log_line in logs {
                        write!(writer, "{}", strip_ansi_codes(log_line))?;
                    }
                } else {
                    for log_line in logs {
                        write!(writer, "{}", log_line)?;
                    }
                }

                writeln!(writer)?;
            }
            if let Some(result) = output.results.get(&key) {
                writeln!(writer, "========================")?;
                writeln!(writer, "Result for {} {}", board.name, board_config.name)?;
                writeln!(writer, "========================")?;

                if strip_ansi {
                    writeln!(writer, "{}", strip_ansi_codes(result))?;
                } else {
                    writeln!(writer, "{}", result)?;
                }
            }
        }
    }

    Ok(())
}
