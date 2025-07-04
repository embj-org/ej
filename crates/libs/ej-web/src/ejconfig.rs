use crate::prelude::*;
use ej_auth::sha256::generate_hash;
use ej_config::{ej_board_config::EjBoardConfigApi, ej_config::EjConfig};
use ej_models::{
    config::{
        ejboard::NewEjBoardDb,
        ejboard_config::{EjBoardConfigDb, NewEjBoardConfigDb},
        ejboard_config_tag::{EjBoardConfigTag, NewEjBoardConfigTag},
        ejconfig::{EjConfigDb, NewEjConfigDb},
        ejtag::{EjTag, NewEjTag},
    },
    db::connection::DbConnection,
};
use tracing::info;
use uuid::Uuid;

pub fn save_config(
    config: EjConfig,
    builder_id: &Uuid,
    conn: &mut DbConnection,
) -> Result<EjConfig> {
    let payload = serde_json::to_string(&config)?;
    let hash = generate_hash(&payload);
    if let Ok(_) = EjConfigDb::fetch_client_config(conn, builder_id, &hash) {
        info!("Config already exists");
        return Ok(config);
    }
    info!("Config with hash {hash} not found for builder {builder_id}. Creating one...");
    let result = config.clone();
    let configdb = NewEjConfigDb::new(*builder_id, config.global.version, hash).save(conn)?;
    for board in config.boards {
        NewEjBoardDb::new(board.id, configdb.id.clone(), board.name, board.description)
            .save(conn)?;
        for board_config in board.configs {
            NewEjBoardConfigDb::new(board_config.id, board.id.clone(), board_config.name)
                .save(conn)?;
            for tag in board_config.tags {
                let tag_db = {
                    if let Ok(tag_db) = EjTag::fetch_by_name(conn, &tag) {
                        tag_db
                    } else {
                        match NewEjTag::new(&tag).save(conn) {
                            Ok(tag_db) => tag_db,
                            Err(err) => {
                                tracing::error!("Failed to create tag {tag}: {err}");
                                continue;
                            }
                        }
                    }
                };
                NewEjBoardConfigTag::new(board_config.id, tag_db.id).save(conn)?;
            }
        }
    }
    Ok(result)
}

pub fn board_config_db_to_board_config_api(
    config_db: EjBoardConfigDb,
    connection: &DbConnection,
) -> Result<EjBoardConfigApi> {
    let tags = EjBoardConfigTag::fetch_by_board_config(config_db.id, connection)?
        .1
        .into_iter()
        .map(|tag| tag.name)
        .collect();

    Ok(EjBoardConfigApi {
        id: config_db.id,
        name: config_db.name,
        tags: tags,
    })
}
