use ej::ej_config::ej_config::{EjConfig, EjUserConfig};
use ej::ej_job::results::api::EjRunOutput;
use std::io::{stderr, stdout};
use std::path::PathBuf;

use crate::build::build;
use crate::logs::dump_logs;
use crate::run::run;
use ej::prelude::*;

pub fn handle_parse(config_path: &PathBuf) -> Result<()> {
    println!("Parsing configuration file: {:?}", config_path);

    let config = EjUserConfig::from_file(config_path)?;

    println!("Configuration parsed successfully");
    println!("Global version: {}", config.global.version);
    println!("Number of boards: {}", config.boards.len());

    for (board_idx, board) in config.boards.iter().enumerate() {
        println!("\nBoard {}: {}", board_idx + 1, board.name);
        println!("  Description: {}", board.description);
        println!("  Configurations: {}", board.configs.len());

        for (config_idx, board_config) in board.configs.iter().enumerate() {
            println!("    Config {}: {}", config_idx + 1, board_config.name);
            println!("      Tags: {:?}", board_config.tags);
            println!("      Build script: {:?}", board_config.build_script);
            println!("      Run script: {:?}", board_config.run_script);
            println!("      Results path: {:?}", board_config.results_path);
            println!("      Library path: {:?}", board_config.library_path);
        }
    }

    Ok(())
}

pub fn handle_run_and_build(config_path: &PathBuf) -> Result<()> {
    println!("Validating configuration file: {:?}", config_path);

    let config = EjUserConfig::from_file(config_path)?;
    let config = EjConfig::from_config(config);
    let mut output = EjRunOutput::new(&config);
    let result = build(&config, &mut output);
    if result.is_ok() {
        dump_logs(&output, stdout())?;
    } else {
        dump_logs(&output, stderr())?;
    }
    result?;
    let result = run(&config, &mut output);
    if result.is_ok() {
        dump_logs(&output, stdout())?;
    } else {
        dump_logs(&output, stderr())?;
    }
    result?;
    Ok(())
}
