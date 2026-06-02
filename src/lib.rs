// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![warn(clippy::all, rust_2018_idioms)]

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[repr(C)]
struct Data {
    pub game_ids: [u32; COUNT],
    pub ghids: [u32; COUNT],
    pub title_offsets: [u32; COUNT + 1],
    pub titles: [u8; DATA_LEN - COUNT * 12 - 4],
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
    let ptr = buf.as_mut_ptr().cast();
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, DATA_LEN) };
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
        let base36 = u32::from_str_radix(id.as_ref(), 36).ok()?;
        DATA.game_ids.binary_search(&base36).ok().map(Self)
    }

    #[inline]
    pub fn ghid(&self) -> Option<u32> {
        let ghid = unsafe { *DATA.ghids.get_unchecked(self.0) };
        (ghid > 0).then_some(ghid)
    }

    #[inline]
    pub fn title(&self) -> &'static str {
        let start = unsafe { *DATA.title_offsets.get_unchecked(self.0) } as usize;
        let end = unsafe { *DATA.title_offsets.get_unchecked(self.0 + 1) } as usize;
        let slice = unsafe { DATA.titles.get_unchecked(start..end) };
        unsafe { std::str::from_utf8_unchecked(slice) }
    }
}
