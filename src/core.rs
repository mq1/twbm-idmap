// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

include!(concat!(env!("OUT_DIR"), "/id_map.rs"));

pub fn get_title(game_id: u32) -> Option<&'static str> {
    TITLE_MAP
        .binary_search_by_key(&game_id, |entry| entry.0)
        .map(|i| TITLE_MAP[i].1)
        .ok()
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<usize> {
    GAMEHACKING_MAP
        .binary_search_by_key(&game_id, |entry| entry.0)
        .map(|i| GAMEHACKING_MAP[i].1 as _)
        .ok()
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: u32) -> Option<&'static str> {
    ASCII_TITLE_MAP
        .binary_search_by_key(&game_id, |entry| entry.0)
        .map(|i| ASCII_TITLE_MAP[i].1)
        .ok()
}
