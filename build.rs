// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{collections::BTreeMap, fs, path::PathBuf};

fn make_title_map<'a>(content: &'a str) -> BTreeMap<u32, &'a str> {
    let mut entries = BTreeMap::new();

    for line in content.lines().skip(1) {
        let (id, title) = line.split_once(" = ").unwrap();
        let id = u32::from_str_radix(id, 36).unwrap();

        entries.insert(id, title);
    }

    entries
}

#[cfg(feature = "ascii-titles")]
fn make_ascii_map<'a>(
    title_map: &'a BTreeMap<u32, &'a str>,
) -> BTreeMap<u32, std::borrow::Cow<'a, str>> {
    let mut entries = BTreeMap::new();

    for (id, title) in title_map {
        let ascii_title = deunicode::deunicode_with_tofu_cow(*title, "");
        if !ascii_title.is_empty() && ascii_title != *title {
            entries.insert(*id, ascii_title);
        }
    }

    entries
}

#[cfg(feature = "gamehacking")]
fn parse_gamehacking_ids() -> BTreeMap<u32, u32> {
    const GHID_ANCHOR: &str = "href=\"/game/";
    const GAMEID_ANCHOR: &str = "<td class=\"text-center\">";

    let mut entries = BTreeMap::new();

    for i in 0..=70 {
        let filename = format!("assets/gamehacking/GameHacking.org - WII - Page {i}.html");
        let content = fs::read_to_string(&filename).unwrap();

        let mut current_slice = &content[..];
        while let Some(ghid_pos) = current_slice.find(GHID_ANCHOR) {
            current_slice = &current_slice[ghid_pos + GHID_ANCHOR.len()..];

            let quote_pos = current_slice.find('"').unwrap();
            let ghid_str = &current_slice[..quote_pos];
            let ghid = ghid_str.parse().unwrap();
            if ghid == 0 {
                continue;
            }

            let gameid_pos = current_slice.find(GAMEID_ANCHOR).unwrap();
            current_slice = &current_slice[gameid_pos + GAMEID_ANCHOR.len()..];
            let td_close_pos = current_slice.find('<').unwrap();
            let gameid_str = current_slice[..td_close_pos].trim();
            let gameid_str_len = gameid_str.len();
            if gameid_str_len != 4 && gameid_str_len != 6 {
                continue;
            }

            let gameid = u32::from_str_radix(gameid_str, 36).unwrap();

            entries.insert(gameid, ghid);
        }
    }

    entries
}

fn encode_u32(value: u32, target_endian: &str) -> [u8; 4] {
    match target_endian {
        "big" => value.to_be_bytes(),
        "little" => value.to_le_bytes(),
        _ => unreachable!(),
    }
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/wiitdb.txt");
    println!("cargo::rerun-if-changed=assets/gamehacking/**");

    let target_endian = std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let titles_txt = fs::read_to_string("assets/wiitdb.txt").unwrap();
    let title_map = make_title_map(&titles_txt);

    #[cfg(feature = "gamehacking")]
    let gamehacking_map = parse_gamehacking_ids();

    #[cfg(feature = "ascii-titles")]
    let ascii_map = make_ascii_map(&title_map);

    let mut bytes = Vec::with_capacity(512 * 1024);

    // title map game ids
    for (id, _title) in &title_map {
        let id_slice = encode_u32(*id, &target_endian);
        bytes.extend_from_slice(&id_slice);
    }

    // title map title offsets
    let mut cursor = 0u32;
    for (_, title) in &title_map {
        let title_offset_slice = encode_u32(cursor, &target_endian);
        bytes.extend_from_slice(&title_offset_slice);

        let len = u32::try_from(title.len()).unwrap();
        cursor = cursor.checked_add(len).unwrap();
    }

    // titles end marker
    let titles_end = encode_u32(cursor, &target_endian);
    bytes.extend_from_slice(&titles_end);

    // gamehacking game ids
    #[cfg(feature = "gamehacking")]
    for (id, _) in &gamehacking_map {
        let id_slice = encode_u32(*id, &target_endian);
        bytes.extend_from_slice(&id_slice);
    }

    // gamehacking ghids
    #[cfg(feature = "gamehacking")]
    for (_, ghid) in &gamehacking_map {
        let ghid_slice = encode_u32(*ghid, &target_endian);
        bytes.extend_from_slice(&ghid_slice);
    }

    // ascii title map game ids
    #[cfg(feature = "ascii-titles")]
    for (id, _) in &ascii_map {
        let id_slice = encode_u32(*id, &target_endian);
        bytes.extend_from_slice(&id_slice);
    }

    // ascii title map title offsets
    #[cfg(feature = "ascii-titles")]
    for (_, ascii_title) in &ascii_map {
        let title_offset_slice = encode_u32(cursor, &target_endian);
        bytes.extend_from_slice(&title_offset_slice);

        let len = u32::try_from(ascii_title.len()).unwrap();
        cursor = cursor.checked_add(len).unwrap();
    }

    // ascii titles end marker
    #[cfg(feature = "ascii-titles")]
    {
        let ascii_titles_end = encode_u32(cursor, &target_endian);
        bytes.extend_from_slice(&ascii_titles_end);
    }

    // titles: [u8]
    for (_, title) in &title_map {
        let title_bytes = title.as_bytes();
        bytes.extend_from_slice(title_bytes);
    }

    // ascii titles: [u8]
    #[cfg(feature = "ascii-titles")]
    for (_, ascii_title) in &ascii_map {
        let title_bytes = ascii_title.as_bytes();
        bytes.extend_from_slice(title_bytes);
    }

    #[allow(unused_mut)]
    let mut meta = format!(
        "const TITLE_COUNT: usize = {}; const TITLES_LEN: usize = {};",
        title_map.len(),
        cursor,
    );

    #[cfg(feature = "gamehacking")]
    meta.push_str(&format!(
        "const GHID_COUNT: usize = {};",
        gamehacking_map.len()
    ));

    #[cfg(feature = "ascii-titles")]
    meta.push_str(&format!(
        "const ASCII_TITLE_COUNT: usize = {};",
        ascii_map.len(),
    ));

    // pad to 4 bytes
    #[cfg(not(feature = "compress"))]
    bytes.resize((bytes.len() + 3) & !3, 0);

    #[cfg(feature = "compress")]
    let bytes = miniz_oxide::deflate::compress_to_vec(&bytes, 9);

    meta.push_str(&format!("const DATA_SIZE: usize = {};", bytes.len()));

    let out_path = out_dir.join("id_map.bin");
    fs::write(out_path, bytes).unwrap();

    let out_path = out_dir.join("id_map_meta.rs");
    fs::write(out_path, meta).unwrap();
}
