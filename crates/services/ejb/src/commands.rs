use ej::ej_job::results::api::EjRunOutput;
use std::io::stdout;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::build::build;
use crate::builder::Builder;
use crate::logs::dump_logs;
use crate::run::run;
use ej::prelude::*;

pub fn handle_parse(builder: &Builder) -> Result<()> {
    let config = &builder.config;

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

pub fn handle_run_and_build(builder: &Builder) -> Result<()> {
    println!("Validating configuration file: {:?}", builder.config_path);

    let config = &builder.config;
    let mut output = EjRunOutput::new(&config);
    let stop = Arc::new(AtomicBool::new(false));
    let result = build(builder, &config, &mut output, Arc::clone(&stop));
    dump_logs(&output, stdout())?;
    result?;
    let result = run(builder, &config, &mut output, Arc::clone(&stop));
    dump_logs(&output, stdout())?;
    result?;
    Ok(())
}
