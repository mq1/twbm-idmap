// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[repr(C)]
pub struct Data {
    game_ids: [u32; COUNT],
    ghids: [u32; COUNT],
    title_offsets: [u32; COUNT + 1],
    titles: [u8; DATA_LEN - COUNT * 12 - 4],
}

impl Data {
    #[inline]
    pub fn game_ids(&self) -> &[u32] {
        &self.game_ids
    }

    #[inline]
    pub fn ghids(&self) -> &[u32] {
        &self.ghids
    }

    #[inline]
    pub fn title_offsets(&self) -> &[u32] {
        &self.title_offsets
    }

    #[inline]
    pub fn titles(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.titles) }
    }
}

pub static DATA: Data = {
    let bytes = *include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));
    unsafe { std::mem::transmute(bytes) }
};
