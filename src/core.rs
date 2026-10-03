// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use rkyv::Archive;

#[cfg(feature = "compress")]
use rkyv::util::AlignedVec;
#[cfg(feature = "compress")]
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[cfg(not(feature = "compress"))]
#[repr(C, align(4))]
struct Align4<T: ?Sized>(T);

#[cfg(not(feature = "compress"))]
static BYTES: Align4<[u8; DATA_LEN]> =
    Align4(*include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin")));

#[cfg(feature = "compress")]
static BYTES: LazyLock<(AlignedVec<4>,)> = LazyLock::new(|| {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    let mut buf = AlignedVec::with_capacity(DATA_LEN);

    miniz_oxide::inflate::decompress_slice_iter_to_slice(
        &mut buf,
        std::iter::once(&compressed[..]),
        false,
        true,
    )
    .unwrap();

    (buf,)
});

#[allow(dead_code)]
#[derive(Archive)]
struct Data {
    title_map: Vec<(u32, String)>,

    #[cfg(feature = "ascii-titles")]
    ascii_title_map: Vec<(u32, String)>,

    #[cfg(feature = "gamehacking")]
    gamehacking_map: Vec<(u32, u32)>,
}

#[must_use]
#[inline]
fn data() -> &'static ArchivedData {
    unsafe { rkyv::access_unchecked(&BYTES.0[..]) }
}

pub fn get_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    data.title_map
        .binary_search_by_key(&game_id, |entry| entry.0.to_native())
        .map(|i| data.title_map[i].1.as_str())
        .ok()
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<usize> {
    let data = data();

    data.gamehacking_map
        .binary_search_by_key(&game_id, |entry| entry.0.to_native())
        .map(|i| data.gamehacking_map[i].1.to_native() as _)
        .ok()
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    data.ascii_title_map
        .binary_search_by_key(&game_id, |entry| entry.0.to_native())
        .map(|i| data.ascii_title_map[i].1.as_str())
        .ok()
}
