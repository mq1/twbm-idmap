// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![warn(clippy::all, rust_2018_idioms)]

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[repr(C)]
struct Data {
    pub game_ids: [u32; COUNT],
    #[cfg(feature = "gamehacking")]
    pub ghids: [Option<std::num::NonZero<u32>>; COUNT],
    pub title_offsets: [u32; COUNT + 1],
    #[cfg(feature = "ascii-titles")]
    pub ascii_title_offsets: [u32; COUNT + 1],
    pub titles: [u8; TITLES_LEN],
}

#[cfg(not(feature = "compress"))]
static DATA: Data = {
    let bytes = *include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));
    unsafe { std::mem::transmute(bytes) }
};

#[cfg(feature = "compress")]
static DATA: std::sync::LazyLock<Box<Data>> = std::sync::LazyLock::new(|| {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    let mut buf = Box::<Data>::new_uninit();

    // inflate
    let ptr = buf.as_mut_ptr().cast::<u8>();
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, std::mem::size_of::<Data>()) };
    let it = std::iter::once(compressed.as_slice());
    miniz_oxide::inflate::decompress_slice_iter_to_slice(slice, it, false, true).unwrap();

    unsafe { buf.assume_init() }
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct GameEntry(usize);

impl GameEntry {
    #[inline]
    pub fn lookup(id: impl AsRef<str>) -> Option<Self> {
        let id = u32::from_str_radix(id.as_ref(), 36).ok()?;
        DATA.game_ids.binary_search(&id).ok().map(Self)
    }

    #[cfg(feature = "gamehacking")]
    #[inline]
    #[must_use]
    pub fn ghid(&self) -> Option<std::num::NonZero<u32>> {
        unsafe { *DATA.ghids.get_unchecked(self.0) }
    }

    #[inline]
    #[must_use]
    pub fn title(&self) -> &'static str {
        #[cfg(not(feature = "compress"))]
        let data = &DATA;

        #[cfg(feature = "compress")]
        let data = DATA.as_ref();

        unsafe {
            let start = *data.title_offsets.get_unchecked(self.0) as usize;
            let end = *data.title_offsets.get_unchecked(self.0 + 1) as usize;
            let slice = data.titles.get_unchecked(start..end);

            std::str::from_utf8_unchecked(slice)
        }
    }

    #[cfg(feature = "ascii-titles")]
    #[inline]
    #[must_use]
    pub fn ascii_title(&self) -> &'static str {
        unsafe {
            let start = *DATA.ascii_title_offsets.get_unchecked(self.0) as usize;
            let end = *DATA.ascii_title_offsets.get_unchecked(self.0 + 1) as usize;

            if start == end {
                self.title()
            } else {
                let slice = DATA.titles.get_unchecked(start..end);
                std::str::from_utf8_unchecked(slice)
            }
        }
    }
}
