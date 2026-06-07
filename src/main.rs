// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![warn(clippy::all, rust_2018_idioms)]

use twbm_idmap::GameEntry;

const USAGE: &str = "Usage: twbm-idmap <GAMEID>";

fn main() {
    let Some(game_id) = std::env::args().nth(1) else {
        eprintln!("{USAGE}");
        std::process::exit(1);
    };

    let Some(entry) = GameEntry::lookup(&game_id) else {
        eprintln!("GameID {game_id} not found");
        std::process::exit(1);
    };

    println!("Title: {}", entry.title());

    #[cfg(feature = "ascii-titles")]
    println!("ASCII Title: {}", entry.ascii_title());

    #[cfg(feature = "gamehacking")]
    println!("GameHacking ID: {:?}", entry.ghid());
}
