// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[repr(C)]
struct Data {
    pub title_map_ids: [u32; TITLE_COUNT],
    pub title_map_title_offsets: [u32; TITLE_COUNT + 1],

    #[cfg(feature = "crc32-hashes")]
    pub hash_map_crc32s: [u32; HASH_COUNT],
    #[cfg(feature = "crc32-hashes")]
    pub hash_map_ids: [u32; HASH_COUNT],

    #[cfg(feature = "gamehacking")]
    pub gamehacking_map_ids: [u32; GHID_COUNT],
    #[cfg(feature = "gamehacking")]
    pub gamehacking_map_ghids: [u32; GHID_COUNT],

    pub titles: [u8; TITLES_LEN],
}

#[cfg(not(feature = "compress"))]
#[repr(C, align(4))]
struct AlignedBytes<T: ?Sized>(T);

#[cfg(not(feature = "compress"))]
static BYTES: &AlignedBytes<[u8]> =
    &AlignedBytes(*include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin")));

#[cfg(not(feature = "compress"))]
#[inline]
fn data() -> &'static Data {
    unsafe { &*BYTES.0.as_ptr().cast::<Data>() }
}

#[cfg(feature = "compress")]
static DATA: std::sync::LazyLock<Box<Data>> = std::sync::LazyLock::new(|| {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    let mut buf = Box::<Data>::new_uninit();

    {
        let slice = unsafe {
            std::slice::from_raw_parts_mut(
                buf.as_mut_ptr().cast::<u8>(),
                std::mem::size_of::<Data>(),
            )
        };

        miniz_oxide::inflate::decompress_slice_iter_to_slice(
            slice,
            std::iter::once(compressed.as_ref()),
            false,
            true,
        )
        .unwrap();
    }

    unsafe { buf.assume_init() }
});

#[cfg(feature = "compress")]
#[inline]
fn data() -> &'static Data {
    DATA.as_ref()
}

pub fn get_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    let idx = data.title_map_ids.binary_search(&game_id).ok()?;

    let start = unsafe { *data.title_map_title_offsets.get_unchecked(idx) } as usize;
    let end = unsafe { *data.title_map_title_offsets.get_unchecked(idx + 1) } as usize;
    let slice = unsafe { data.titles.get_unchecked(start..end) };
    let title = unsafe { std::str::from_utf8_unchecked(slice) };

    Some(title)
}

#[cfg(feature = "crc32-hashes")]
pub fn is_crc32_hash_known(game_id: u32, hash: u32) -> bool {
    let data = data();

    let Ok(idx) = data.hash_map_crc32s.binary_search(&hash) else {
        return false;
    };

    let known_id = unsafe { *data.hash_map_ids.get_unchecked(idx) };

    known_id == game_id
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<u32> {
    let data = data();

    let idx = data.gamehacking_map_ids.binary_search(&game_id).ok()?;

    let ghid = unsafe { *data.gamehacking_map_ghids.get_unchecked(idx) };

    Some(ghid)
}
