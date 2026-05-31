// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![warn(clippy::all, rust_2018_idioms)]

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

use std::{borrow::Cow, num::NonZeroU32};

#[repr(align(4))]
struct Data([u8; DATA_LEN]);

impl Data {
    #[inline]
    fn game_ids(&self) -> &[u32] {
        let ptr = self.0.as_ptr().cast::<u32>();
        unsafe { std::slice::from_raw_parts(ptr, COUNT) }
    }

    #[inline]
    fn ghids(&self) -> &[u32] {
        let ptr = self.0.as_ptr().cast::<u32>();
        unsafe { std::slice::from_raw_parts(ptr.add(COUNT), COUNT) }
    }

    #[inline]
    fn title_offsets(&self) -> &[u32] {
        let ptr = self.0.as_ptr().cast::<u32>();
        unsafe { std::slice::from_raw_parts(ptr.add(COUNT * 2), COUNT + 1) }
    }

    #[inline]
    fn titles(&self) -> &str {
        let slice = unsafe { self.0.get_unchecked(COUNT * 12 + 4..DATA_LEN) };
        unsafe { std::str::from_utf8_unchecked(slice) }
    }
}

static DATA: Data = Data(*include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin")));

#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct GameID(u32);

impl From<u32> for GameID {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<[u8; 6]> for GameID {
    fn from(value: [u8; 6]) -> Self {
        let s = unsafe { std::str::from_utf8_unchecked(&value) };
        GameID::from(s)
    }
}

impl From<&str> for GameID {
    fn from(value: &str) -> Self {
        u32::from_str_radix(value, 36)
            .map(GameID)
            .unwrap_or_default()
    }
}

impl From<String> for GameID {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for GameID {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl<'a> From<Cow<'a, str>> for GameID {
    fn from(value: Cow<'a, str>) -> Self {
        Self::from(value.as_ref())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GameEntry(usize);

impl GameEntry {
    #[inline]
    pub fn lookup(id: impl Into<GameID>) -> Option<Self> {
        let base36 = id.into().0;
        DATA.game_ids().binary_search(&base36).ok().map(GameEntry)
    }

    #[inline]
    pub fn ghid(&self) -> Option<NonZeroU32> {
        let ghid = unsafe { *DATA.ghids().get_unchecked(self.0) };
        NonZeroU32::new(ghid)
    }

    #[inline]
    pub fn title(&self) -> &'static str {
        let start = unsafe { *DATA.title_offsets().get_unchecked(self.0) } as usize;
        let end = unsafe { *DATA.title_offsets().get_unchecked(self.0 + 1) } as usize;
        unsafe { DATA.titles().get_unchecked(start..end) }
    }
}
