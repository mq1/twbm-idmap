// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![warn(clippy::all, rust_2018_idioms)]

const USAGE: &str = "Usage: twbm-idmap <GAMEID>";

fn main() {
    let Some(game_id) = std::env::args().nth(1) else {
        eprintln!("{USAGE}");
        std::process::exit(1);
    };

    let title = twbm_idmap::get_title(&game_id);
    println!("Title: {title:?}");

    #[cfg(feature = "gamehacking")]
    {
        let ghid = twbm_idmap::get_ghid(&game_id);
        println!("GameHacking ID: {ghid:?}");
    }

    #[cfg(feature = "hashes")]
    {
        let hashes = twbm_idmap::get_crc32_hashes(&game_id).map(|hashes| {
            hashes
                .iter()
                .map(|hash| format!("{hash:08x}"))
                .collect::<Vec<_>>()
        });
        println!("CRC32 hashes: {hashes:?}");
    }
}
