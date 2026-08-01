// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

include!(concat!(env!("OUT_DIR"), "/id_map.rs"));

pub fn get_title(game_id: u32) -> Option<&'static str> {
    let idx = TITLE_MAP
        .binary_search_by_key(&game_id, |entry| entry.0)
        .ok()?;
    let title_idx = TITLE_MAP[idx].1 as usize;
    let title = &ALL_TITLES[title_idx];

    Some(title)
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<usize> {
    let idx = GAMEHACKING_MAP
        .binary_search_by_key(&game_id, |entry| entry.0)
        .ok()?;
    let ghid = GAMEHACKING_MAP[idx].1 as usize;

    Some(ghid)
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: u32) -> Option<&'static str> {
    let idx = ASCII_TITLE_MAP
        .binary_search_by_key(&game_id, |entry| entry.0)
        .ok()?;
    let title_idx = ASCII_TITLE_MAP[idx].1 as usize;
    let title = &ALL_TITLES[title_idx];

    Some(title)
}
