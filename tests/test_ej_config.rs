use std::{error::Error, path::Path};

use common::{EJD, test_context::TestContext};
use diesel::prelude::*;
use ej::ej_config::{
    ej_board::EjBoard,
    ej_board_config::EjBoardConfig,
    ej_config::{EjConfig, EjGlobalConfig},
};

mod common;

fn setup_database(connection: &mut PgConnection) {
    const QUERIES: [&str; 3] = [
        "INSERT INTO permission (id) VALUES ('builder.create');",
        "INSERT INTO ejclient (name, hash) VALUES ('root', '$argon2id$v=19$m=19456,t=2,p=1$C4ZIwZW3k3Lec4ml/LlXhg$o5GwzCUQKicsKHfJvLAyL2GSyx9topN12vgZA4avM+g');",
        "INSERT INTO client_permission (ejclient_id, permission_id) VALUES ((SELECT id FROM ejclient WHERE name = 'root'), 'builder.create')",
    ];
    for query in QUERIES {
        let query = diesel::sql_query(query);
        query
            .execute(connection)
            .expect("Couldn't insert values to database");
    }
}

#[tokio::test]
async fn test_from_file() -> Result<(), Box<dyn Error>> {
    let rpi3_board_configs: Vec<EjBoardConfig> = vec![
        EjBoardConfig {
            description: String::from("Rpi3 Wayland"),
            tags: vec![String::from("wayland"), String::from("arm64")],
            build_script: String::from("/home/work/rpi/wayland/scripts/build_rpi4_wayland.sh"),
            run_script: String::from("/home/work/rpi/wayland/scripts/run_rpi4_wayland.sh"),
            results_path: String::from("/home/work/rpi/wayland/results/results.json"),
            library_path: String::from("/home/work/rpi/wayland/lib"),
        },
        EjBoardConfig {
            description: String::from("Rpi3 SDL"),
            tags: vec![String::from("sdl2"), String::from("arm64")],
            build_script: String::from("/home/work/rpi/sdl/scripts/build_rpi4_wayland.sh"),
            run_script: String::from("/home/work/rpi/sdl/scripts/run_rpi4_wayland.sh"),
            results_path: String::from("/home/work/rpi/sdl/results/results.json"),
            library_path: String::from("/home/work/rpi/sdl/lib"),
        },
    ];
    let x86_configs: Vec<EjBoardConfig> = vec![
        EjBoardConfig {
            description: String::from("Wayland build for desktop"),
            tags: vec![String::from("wayland"), String::from("x86_64")],
            build_script: String::from("/home/work/x86/wayland/scripts/build_rpi4_wayland.sh"),
            run_script: String::from("/home/work/x86/wayland/scripts/run_rpi4_wayland.sh"),
            results_path: String::from("/home/work/x86/wayland/results/results.json"),
            library_path: String::from("/home/work/x86/wayland/lib"),
        },
        EjBoardConfig {
            description: String::from("X11 build for desktop"),
            tags: vec![String::from("x11"), String::from("x86_64")],
            build_script: String::from("/home/work/x86/x11/scripts/build_rpi4_wayland.sh"),
            run_script: String::from("/home/work/x86/x11/scripts/run_rpi4_wayland.sh"),
            results_path: String::from("/home/work/x86/x11/results/results.json"),
            library_path: String::from("/home/work/x86/x11/lib"),
        },
    ];
    let boards: Vec<EjBoard> = vec![
        EjBoard {
            name: String::from("Raspberry Pi 3"),
            description: String::from("Raspberry Pi 3 Model B+"),
            configs: rpi3_board_configs,
        },
        EjBoard {
            name: String::from("x86 PC running Fedora 41"),
            description: String::from("AMD Ryzen 5 3600 desktop with NVIDIA GTX 1650"),
            configs: x86_configs,
        },
    ];

    let expected_config = EjConfig {
        global: EjGlobalConfig {
            version: String::from("1.0.0"),
        },
        boards,
    };

    let config = EjConfig::from_file(Path::new("examples/config.toml"))?;
    assert_eq!(config, expected_config);
    Ok(())
}

#[tokio::test]
async fn test_send_to_server() {
    let (mut db, client) = TestContext::from_env();
    setup_database(&mut db.conn);

    let config =
        EjConfig::from_file(Path::new("examples/config.toml")).expect("parsing from file failed");
    let payload = serde_json::to_string(&config).expect("json serialization failed");

    let post_result: EjConfig = EJD
        .post(&client, "builder/config", payload)
        .await
        .expect("post failed");
    assert_eq!(post_result, config);
}
