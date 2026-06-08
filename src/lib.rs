// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![warn(clippy::all, rust_2018_idioms)]

mod core;

pub fn get_title(game_id: impl AsRef<str>) -> Option<&'static str> {
    let game_id = u32::from_str_radix(game_id.as_ref(), 36).ok()?;
    core::get_title(game_id)
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: impl AsRef<str>) -> Option<u32> {
    let game_id = u32::from_str_radix(game_id.as_ref(), 36).ok()?;
    core::get_ghid(game_id)
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: impl AsRef<str>) -> Option<&'static str> {
    let game_id = u32::from_str_radix(game_id.as_ref(), 36).ok()?;
    core::get_ascii_title(game_id).or_else(|| core::get_title(game_id))
}
