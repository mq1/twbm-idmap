// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[repr(C, align(4))]
struct Align4<const N: usize>([u8; N]);

#[cfg(not(feature = "_compress"))]
static BYTES: Align4<DATA_SIZE> = Align4(*include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin")));

#[cfg(feature = "_compress")]
static BYTES: std::sync::LazyLock<Box<Align4<DATA_SIZE>>> = std::sync::LazyLock::new(|| {
    const COMPRESSED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    let mut buf = unsafe { Box::<Align4<DATA_SIZE>>::new_zeroed().assume_init() };

    #[cfg(feature = "compress-deflate")]
    miniz_oxide::inflate::decompress_slice_iter_to_slice(
        &mut buf.0[..],
        std::iter::once(COMPRESSED),
        false,
        true,
    )
    .unwrap();

    #[cfg(feature = "compress-zstd")]
    unsafe {
        let written = zstd_sys::ZSTD_decompress(
            buf.0.as_mut_ptr().cast(),
            DATA_SIZE,
            COMPRESSED.as_ptr().cast(),
            COMPRESSED.len(),
        );

        debug_assert_eq!(zstd_sys::ZSTD_isError(written), 0);
        debug_assert_eq!(written, DATA_SIZE);
    }

    buf
});

#[inline]
fn data() -> &'static Data {
    unsafe { BYTES.0.as_ptr().cast::<Data>().as_ref_unchecked() }
}

pub fn get_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    let idx = data.title_map_ids.binary_search(&game_id).ok()?;

    let title_str = unsafe {
        let start = *data.title_map_offsets.get_unchecked(idx) as usize;
        let end = *data.title_map_offsets.get_unchecked(idx.unchecked_add(1)) as usize;

        let title_slice = data.all_titles.get_unchecked(start..end);
        str::from_utf8_unchecked(title_slice)
    };

    Some(title_str)
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<usize> {
    let data = data();

    let idx = data.gamehacking_map_ids.binary_search(&game_id).ok()?;
    let ghid = unsafe { *data.gamehacking_map_ghids.get_unchecked(idx) } as usize;

    Some(ghid)
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: u32) -> Option<&'static str> {
    let data = data();

    let idx = data.ascii_title_map_ids.binary_search(&game_id).ok()?;

    let title_str = unsafe {
        let start = *data.ascii_title_map_offsets.get_unchecked(idx) as usize;
        let end = *data
            .ascii_title_map_offsets
            .get_unchecked(idx.unchecked_add(1)) as usize;

        let title_slice = data.all_titles.get_unchecked(start..end);
        str::from_utf8_unchecked(title_slice)
    };

    Some(title_str)
}
