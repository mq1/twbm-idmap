// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![warn(clippy::all, rust_2018_idioms)]

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[repr(C)]
struct Data {
    pub title_map_ids: [u32; TITLE_COUNT],
    pub title_map_title_offsets: [u32; TITLE_COUNT + 1],

    #[cfg(feature = "gamehacking")]
    pub gamehacking_map_ids: [u32; GHID_COUNT],
    #[cfg(feature = "gamehacking")]
    pub gamehacking_map_ghids: [u32; GHID_COUNT],

    #[cfg(feature = "ascii-titles")]
    pub ascii_title_map_ids: [u32; ASCII_TITLE_COUNT],
    #[cfg(feature = "ascii-titles")]
    pub ascii_title_map_title_offsets: [u32; ASCII_TITLE_COUNT + 1],

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

    // inflate directly into the buffer
    let ptr = buf.as_mut_ptr().cast::<u8>();
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, std::mem::size_of::<Data>()) };
    let it = std::iter::once(compressed.as_slice());
    miniz_oxide::inflate::decompress_slice_iter_to_slice(slice, it, false, true).unwrap();

    unsafe { buf.assume_init() }
});

fn _get_title(game_id: u32) -> Option<&'static str> {
    let index = DATA.title_map_ids.binary_search(&game_id).ok()?;

    unsafe {
        let start = *DATA.title_map_title_offsets.get_unchecked(index) as usize;
        let end = *DATA.title_map_title_offsets.get_unchecked(index + 1) as usize;
        let slice = DATA.titles.get_unchecked(start..end);
        Some(std::str::from_utf8_unchecked(slice))
    }
}

pub fn get_title(game_id: impl AsRef<str>) -> Option<&'static str> {
    let game_id = u32::from_str_radix(game_id.as_ref(), 36).ok()?;
    _get_title(game_id)
}

#[cfg(feature = "gamehacking")]
fn _get_ghid(game_id: u32) -> Option<u32> {
    let idx = DATA.gamehacking_map_ids.binary_search(&game_id).ok()?;
    let ghid = unsafe { *DATA.gamehacking_map_ghids.get_unchecked(idx) };
    Some(ghid)
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: impl AsRef<str>) -> Option<u32> {
    let game_id = u32::from_str_radix(game_id.as_ref(), 36).ok()?;
    _get_ghid(game_id)
}

#[cfg(feature = "ascii-titles")]
fn _get_ascii_title(game_id: u32) -> Option<&'static str> {
    let idx = DATA.ascii_title_map_ids.binary_search(&game_id).ok()?;

    unsafe {
        let start = *DATA.ascii_title_map_title_offsets.get_unchecked(idx) as usize;
        let end = *DATA.ascii_title_map_title_offsets.get_unchecked(idx + 1) as usize;
        let slice = DATA.titles.get_unchecked(start..end);
        Some(std::str::from_utf8_unchecked(slice))
    }
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: impl AsRef<str>) -> Option<&'static str> {
    let game_id = u32::from_str_radix(game_id.as_ref(), 36).ok()?;
    _get_ascii_title(game_id).or_else(|| _get_title(game_id))
}
