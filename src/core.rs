// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeMap;

include!(concat!(env!("OUT_DIR"), "/id_map.rs"));

#[derive(rkyv::Archive)]
#[allow(unused)]
struct Data {
    title_map: BTreeMap<u32, usize>,

    #[cfg(feature = "gamehacking")]
    gamehacking_map: BTreeMap<u32, usize>,

    #[cfg(feature = "ascii-titles")]
    ascii_title_map: BTreeMap<u32, usize>,

    all_titles: Vec<String>,
}

#[cfg(not(feature = "compress"))]
#[repr(C, align(4))]
struct Aligned4<T: ?Sized>(T);

#[cfg(not(feature = "compress"))]
static BYTES: Aligned4<[u8; DATA_SIZE]> =
    Aligned4(*include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin")));

#[cfg(feature = "compress")]
static BYTES: std::sync::LazyLock<(rkyv::util::AlignedVec<4>,)> = std::sync::LazyLock::new(|| {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    let mut buf = rkyv::util::AlignedVec::with_capacity(DATA_SIZE);
    unsafe { buf.set_len(DATA_SIZE) };

    // inflate directly into the buffer
    let it = std::iter::once(&compressed[..]);
    miniz_oxide::inflate::decompress_slice_iter_to_slice(&mut buf, it, false, true).unwrap();

    (buf,)
});

#[inline]
fn data() -> &'static ArchivedData {
    unsafe { rkyv::access_unchecked(&BYTES.0) }
}

pub fn get_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    let idx = data.title_map.get(&game_id.into())?.to_native() as usize;
    let title = data.all_titles[idx].as_str();

    Some(title)
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<usize> {
    let data = data();

    let idx = data.gamehacking_map.get(&game_id.into())?.to_native() as usize;

    Some(idx)
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    let idx = data.ascii_title_map.get(&game_id.into())?.to_native() as usize;
    let title = data.all_titles[idx].as_str();

    Some(title)
}
