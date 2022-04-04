use std::path::PathBuf;
use std::thread;

use anyhow::anyhow;
use clap::Parser;
use notify_rust::Notification;

mod config;
use config::Config;
mod ogs;
use ogs::{Game, OgsAgent};
mod state;
use state::State;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long)]
    config_file: Option<PathBuf>,
}

fn main_with_config(config: Config) -> anyhow::Result<()> {
    let agent = OgsAgent::new(&config.cookie_file)?;

    let logged_in_user = agent.logged_in_user()?;
    let active_games = agent.active_games()?;
    let games_awaiting_move = active_games
        .into_iter()
        .filter(|game| game.json.clock.current_player == logged_in_user.id)
        .collect();

    let mut state = State {
        logged_in_user,
        games_awaiting_move,
    };

    let mut ogs_icon = config.icon_dir.clone();
    ogs_icon.push("ogs_icon.png");
    for game in &state.games_awaiting_move {
        Notification::new()
            .summary("ogs-notify")
            .body(&format!(
                "It's your move against {}",
                game.other_user(&state.logged_in_user).username,
            ))
            .icon(
                ogs_icon
                    .to_str()
                    .ok_or_else(|| anyhow!("Could not get string version of path."))?,
            )
            .show()
            .unwrap();
    }

    loop {
        thread::sleep(config.check_interval);
        let active_games = agent.active_games()?;
        let games_awaiting_move: Vec<Game> = active_games
            .into_iter()
            .filter(|game| game.json.clock.current_player == state.logged_in_user.id)
            .collect();

        for g1 in &games_awaiting_move {
            if state
                .games_awaiting_move
                .iter()
                .find(|g2| g1.id == g2.id)
                .is_none()
            {
                Notification::new()
                    .summary("ogs-notify")
                    .body(&format!(
                        "It's your move against {}",
                        g1.other_user(&state.logged_in_user).username,
                    ))
                    .icon(
                        ogs_icon
                            .to_str()
                            .ok_or_else(|| anyhow!("Could not get string version of path."))?,
                    )
                    .show()
                    .unwrap();
            }
        }

        state.games_awaiting_move = games_awaiting_move;
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::try_from(args)?;

    main_with_config(config)
}
