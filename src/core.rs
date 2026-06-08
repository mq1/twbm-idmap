// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

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
const BYTES: &[u8; DATA_SIZE] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

#[cfg(feature = "compress")]
static BYTES: std::sync::LazyLock<Box<[u8]>> = std::sync::LazyLock::new(|| {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    let mut buf = Box::new_uninit_slice(DATA_SIZE);

    // inflate directly into the buffer
    let ptr = buf.as_mut_ptr().cast::<u8>();
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, DATA_SIZE) };
    let it = std::iter::once(compressed.as_ref());
    miniz_oxide::inflate::decompress_slice_iter_to_slice(slice, it, false, true).unwrap();

    unsafe { buf.assume_init() }
});

#[inline]
fn data() -> &'static Data {
    let ptr = BYTES.as_ptr().cast::<Data>();
    unsafe { ptr.as_ref_unchecked() }
}

pub fn get_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    let idx = data.title_map_ids.binary_search(&game_id).ok()?;

    unsafe {
        let start = *data.title_map_title_offsets.get_unchecked(idx) as usize;
        let end = *data.title_map_title_offsets.get_unchecked(idx + 1) as usize;
        let slice = data.titles.get_unchecked(start..end);
        Some(std::str::from_utf8_unchecked(slice))
    }
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<u32> {
    let data = data();

    let idx = data.gamehacking_map_ids.binary_search(&game_id).ok()?;

    let ghid = unsafe { *data.gamehacking_map_ghids.get_unchecked(idx) };
    Some(ghid)
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    let idx = data.ascii_title_map_ids.binary_search(&game_id).ok()?;

    unsafe {
        let start = *data.ascii_title_map_title_offsets.get_unchecked(idx) as usize;
        let end = *data.ascii_title_map_title_offsets.get_unchecked(idx + 1) as usize;
        let slice = data.titles.get_unchecked(start..end);
        Some(std::str::from_utf8_unchecked(slice))
    }
}
