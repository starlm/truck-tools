/*
 * The game's documents directory can be in different places on Linux depending on how the game is ran.
 * Since the documents directory isn't guaranteed to be the same for both ETS2 and ATS,
 * the GUI should provide a way to set the documents directory on a per-game basis or provide an "automatic" mode.
 *
 * Scenarios:
 *   a) User installed the Linux build of the game
 *      -> ETS2 & ATS are both writing to $HOME/Documents
 *   b) User installed the Windows build of the game
 *      -> ETS2/ATS are writing to their own WINE prefixes
 *         $XDG_DATA_HOME/Steam/steamapps/compatdata/<APP_ID>/pfx/drive_c/users/steamuser/Documents
 *
 * Scenario B is the most likely since TruckersMP and (most) job trackers only support Windows build
 */

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    ETS2,
    ATS,
}

const ETS2_APPID: u32 = 227300;
const ATS_APPID: u32 = 270880;

fn get_game_name<'a>(game: &Game) -> &'a str {
    match game {
        Game::ETS2 => "Euro Truck Simulator 2",
        Game::ATS => "American Truck Simulator",
    }
}

#[tauri::command]
pub async fn linux_get_game_docs(game: Game) -> Result<PathBuf, &'static str> {
    let xdg_data_dir: PathBuf = match std::env::var("XDG_DATA_HOME") {
        Ok(dir) => dir.into(),
        // XDG_DATA_HOME isn't guaranteed to be set
        Err(_) => match std::env::home_dir() {
            Some(dir) => dir.join(".local/share"),
            None => return Err("Failed to resolve $XDG_DATA_HOME"),
        },
    };
    let steamapps_dir = xdg_data_dir.join("Steam/steamapps");

    if !steamapps_dir.exists() {
        return Err("Failed to Steam directory");
    }

    // Check if the game is native
    let game_install_dir = steamapps_dir.join("common").join(get_game_name(&game));
    if !game_install_dir.exists() {
        return Err("Game is not installed");
    }

    if game_install_dir.join("bin/linux_x64").exists() {
        Ok(match std::env::home_dir() {
            Some(dir) => dir.join("Documents"),
            None => return Err("Failed to resolve $HOME"),
        })
    } else {
        let appid = match game {
            Game::ETS2 => ETS2_APPID,
            Game::ATS => ATS_APPID,
        };

        let docs_dir = steamapps_dir
            .join("compatdata")
            .join(appid.to_string())
            .join("pfx/drive_c/users/steamuser/Documents");

        Ok(docs_dir)
    }
}
