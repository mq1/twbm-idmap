// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![warn(clippy::all, rust_2018_idioms)]

#[cfg(not(feature = "compress"))]
mod uncompressed;

#[cfg(not(feature = "compress"))]
use uncompressed::DATA;

#[cfg(feature = "compress")]
mod compressed;

#[cfg(feature = "compress")]
use compressed::DATA;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GameEntry(usize);

impl GameEntry {
    #[inline]
    pub fn lookup(id: impl AsRef<str>) -> Option<Self> {
        let base36 = u32::from_str_radix(id.as_ref(), 36).ok()?;
        DATA.game_ids().binary_search(&base36).ok().map(Self)
    }

    #[inline]
    pub fn ghid(&self) -> Option<u32> {
        let ghid = unsafe { *DATA.ghids().get_unchecked(self.0) };
        (ghid > 0).then_some(ghid)
    }

    #[inline]
    pub fn title(&self) -> &'static str {
        let start = unsafe { *DATA.title_offsets().get_unchecked(self.0) } as usize;
        let end = unsafe { *DATA.title_offsets().get_unchecked(self.0 + 1) } as usize;
        unsafe { DATA.titles().get_unchecked(start..end) }
    }
}
