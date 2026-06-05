// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{fs, path::PathBuf};

struct GameEntry<'a> {
    id: u32,
    #[cfg(feature = "gamehacking")]
    ghid: u32,
    title: &'a str,
}

fn make_id_map(content: &str) -> Vec<GameEntry<'_>> {
    let mut entries = Vec::with_capacity(16 * 1024);

    for line in content.lines().skip(1) {
        let (id, title) = line.split_once(" = ").unwrap();
        let id = u32::from_str_radix(id, 36).unwrap();

        entries.push(GameEntry {
            id,
            #[cfg(feature = "gamehacking")]
            ghid: 0,
            title,
        });
    }

    entries.sort_by_key(|e| e.id);
    entries
}

#[cfg(feature = "gamehacking")]
fn parse_gamehacking_ids(entries: &mut [GameEntry]) {
    const GHID_ANCHOR: &str = "href=\"/game/";
    const GAMEID_ANCHOR: &str = "<td class=\"text-center\">";

    for i in 0..=70 {
        let filename = format!("assets/gamehacking/GameHacking.org - WII - Page {i}.html");
        let content = fs::read_to_string(&filename).unwrap();

        let mut current_slice = &content[..];
        while let Some(ghid_pos) = current_slice.find(GHID_ANCHOR) {
            current_slice = &current_slice[ghid_pos + GHID_ANCHOR.len()..];

            let quote_pos = current_slice.find('"').unwrap();
            let ghid_str = &current_slice[..quote_pos];
            let ghid = ghid_str.parse().unwrap();

            let gameid_pos = current_slice.find(GAMEID_ANCHOR).unwrap();
            current_slice = &current_slice[gameid_pos + GAMEID_ANCHOR.len()..];
            let td_close_pos = current_slice.find('<').unwrap();
            let gameid_str = current_slice[..td_close_pos].trim();
            if !matches!(gameid_str.len(), 4 | 6) {
                continue;
            }
            let gameid = u32::from_str_radix(gameid_str, 36).unwrap();

            if let Ok(i) = entries.binary_search_by_key(&gameid, |e| e.id) {
                entries[i].ghid = ghid;
            }
        }
    }
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

    #[cfg_attr(not(feature = "gamehacking"), allow(unused_mut))]
    let mut entries = make_id_map(&titles_txt);

    #[cfg(feature = "gamehacking")]
    parse_gamehacking_ids(&mut entries);

    let mut bytes = Vec::with_capacity(512 * 1024);

    // first the ids
    for entry in &entries {
        let slice = encode_u32(entry.id, &target_endian);
        bytes.extend_from_slice(&slice);
    }

    // then the ghids
    #[cfg(feature = "gamehacking")]
    for entry in &entries {
        let slice = encode_u32(entry.ghid, &target_endian);
        bytes.extend_from_slice(&slice);
    }

    // then the title offsets
    let mut cursor = 0u32;
    for entry in &entries {
        let slice = encode_u32(cursor, &target_endian);
        bytes.extend_from_slice(&slice);
        let len = u32::try_from(entry.title.len()).unwrap();
        cursor = cursor.checked_add(len).unwrap();
    }

    // then TITLES_LEN as the last offset
    let slice = encode_u32(cursor, &target_endian);
    bytes.extend_from_slice(&slice);

    // then the titles
    for entry in &entries {
        let slice = entry.title.as_bytes();
        bytes.extend_from_slice(slice);
    }

    let meta = format!(
        "const COUNT: usize = {}; const TITLES_LEN: usize = {};",
        entries.len(),
        cursor
    );

    // pad to 4 bytes
    #[cfg(not(feature = "compress"))]
    bytes.resize((bytes.len() + 3) & !3, 0);

    #[cfg(feature = "compress")]
    let bytes = miniz_oxide::deflate::compress_to_vec(&bytes, 9);

    let out_path = out_dir.join("id_map.bin");
    fs::write(out_path, bytes).unwrap();

    let out_path = out_dir.join("id_map_meta.rs");
    fs::write(out_path, meta).unwrap();
}
