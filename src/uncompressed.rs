// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[repr(align(4))]
pub struct Data([u8; DATA_LEN]);

impl Data {
    #[inline]
    pub fn game_ids(&self) -> &[u32] {
        let ptr = self.0.as_ptr().cast::<u32>();
        unsafe { std::slice::from_raw_parts(ptr, COUNT) }
    }

    #[inline]
    pub fn ghids(&self) -> &[u32] {
        let ptr = self.0.as_ptr().cast::<u32>();
        unsafe { std::slice::from_raw_parts(ptr.add(COUNT), COUNT) }
    }

    #[inline]
    pub fn title_offsets(&self) -> &[u32] {
        let ptr = self.0.as_ptr().cast::<u32>();
        unsafe { std::slice::from_raw_parts(ptr.add(COUNT * 2), COUNT + 1) }
    }

    #[inline]
    pub fn titles(&self) -> &str {
        let slice = unsafe { self.0.get_unchecked(COUNT * 12 + 4..DATA_LEN) };
        unsafe { std::str::from_utf8_unchecked(slice) }
    }
}

pub static DATA: Data = Data(*include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin")));
