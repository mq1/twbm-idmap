// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeMap;

#[cfg(feature = "compress")]
include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[derive(rkyv::Archive, rkyv::Deserialize)]
struct Game {
    title: String,

    #[cfg(feature = "hashes")]
    crc32s: Vec<u32>,
}

#[derive(rkyv::Archive, rkyv::Deserialize)]
struct Data {
    title_map: BTreeMap<u32, Game>,

    #[cfg(feature = "gamehacking")]
    gamehacking_map: BTreeMap<u32, u32>,
}

#[cfg(not(feature = "compress"))]
#[repr(C, align(4))]
struct AlignedBytes<T: ?Sized>(T);

#[cfg(not(feature = "compress"))]
static BYTES: &AlignedBytes<[u8]> =
    &AlignedBytes(*include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin")));

#[cfg(not(feature = "compress"))]
#[inline]
fn data() -> &'static ArchivedData {
    unsafe { rkyv::access_unchecked(&BYTES.0) }
}

#[cfg(feature = "compress")]
static BYTES: std::sync::LazyLock<rkyv::util::AlignedVec<4>> = std::sync::LazyLock::new(|| {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    let mut buf = rkyv::util::AlignedVec::with_capacity(DATA_SIZE);
    unsafe { buf.set_len(DATA_SIZE) }

    miniz_oxide::inflate::decompress_slice_iter_to_slice(
        buf.as_mut_slice(),
        std::iter::once(compressed.as_ref()),
        false,
        true,
    )
    .unwrap();

    buf
});

#[cfg(feature = "compress")]
#[inline]
fn data() -> &'static ArchivedData {
    unsafe { rkyv::access_unchecked(&BYTES) }
}

pub fn get_title(game_id: u32) -> Option<&'static str> {
    data()
        .title_map
        .get(&game_id.into())
        .map(|game| game.title.as_str())
}

#[cfg(feature = "hashes")]
pub fn is_crc32_hash_known(game_id: u32, hash: u32) -> bool {
    data()
        .title_map
        .get(&game_id.into())
        .is_some_and(|game| game.crc32s.contains(&hash.into()))
}

#[cfg(feature = "hashes")]
pub fn get_crc32_hashes(game_id: u32) -> Option<&'static [u32]> {
    data().title_map.get(&game_id.into()).map(|game| {
        let slice = game.crc32s.as_slice();
        unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<u32>(), slice.len()) }
    })
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<u32> {
    data().gamehacking_map.get(&game_id.into()).map(Into::into)
}
