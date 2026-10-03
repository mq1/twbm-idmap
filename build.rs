// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use rkyv::{Archive, Serialize};
use std::{fs, path::PathBuf};

#[derive(Archive, Serialize)]
struct Data {
    title_map: Vec<(u32, String)>,

    #[cfg(feature = "ascii-titles")]
    ascii_title_map: Vec<(u32, String)>,

    #[cfg(feature = "gamehacking")]
    gamehacking_map: Vec<(u32, u32)>,
}

fn parse_titles_txt(content: &str) -> Vec<(u32, String)> {
    let mut entries = Vec::with_capacity(16_384);

    for line in content.lines().skip(1) {
        let (id, title) = line.split_once(" = ").unwrap();
        let id = u32::from_str_radix(id, 36).unwrap();

        entries.push((id, title.to_string()));
    }

    entries
}

#[cfg(feature = "ascii-titles")]
fn make_ascii_map(
    title_map: &[(u32, String)],
    en_title_map: Vec<(u32, String)>,
) -> Vec<(u32, String)> {
    let mut entries = Vec::with_capacity(4096);

    for ((id, title), (en_id, en_title)) in title_map.iter().zip(en_title_map) {
        assert_eq!(*id, en_id);

        if title.is_ascii() {
            // original title is already ascii, don't add an entry
            // we handle this by falling back to the original title
            continue;
        }

        if en_title.is_ascii() {
            entries.push((en_id, en_title));
        } else {
            let mut ascii_title = en_title
                .chars()
                .filter(char::is_ascii)
                .skip_while(char::is_ascii_whitespace)
                .collect::<String>();

            ascii_title.truncate(ascii_title.trim_end().len());

            assert!(!ascii_title.is_empty());
            entries.push((en_id, ascii_title.to_string()));
        }
    }

    entries
}

#[cfg(feature = "gamehacking")]
fn parse_gamehacking_ids() -> Vec<(u32, u32)> {
    const GHID_ANCHOR: &str = "href=\"/game/";
    const GAMEID_ANCHOR: &str = "<td class=\"text-center\">";

    let mut entries = Vec::with_capacity(2048);

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

            entries.push((gameid, ghid));
        }
    }

    entries.sort_unstable_by_key(|(id, _)| *id);
    entries.dedup_by_key(|(id, _)| *id);

    entries
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/**");

    let mut title_map = parse_titles_txt(&fs::read_to_string("assets/wiitdb.txt").unwrap());

    #[cfg(feature = "ascii-titles")]
    let mut ascii_title_map = make_ascii_map(
        &title_map,
        parse_titles_txt(&fs::read_to_string("assets/wiitdb-en.txt").unwrap()),
    );

    title_map.sort_unstable_by_key(|(id, _)| *id);
    title_map.dedup_by_key(|(id, _)| *id);

    #[cfg(feature = "ascii-titles")]
    {
        ascii_title_map.sort_unstable_by_key(|(id, _)| *id);
        ascii_title_map.dedup_by_key(|(id, _)| *id);
    }

    #[cfg(feature = "gamehacking")]
    let gamehacking_map = parse_gamehacking_ids();

    let data = Data {
        title_map,

        #[cfg(feature = "ascii-titles")]
        ascii_title_map,

        #[cfg(feature = "gamehacking")]
        gamehacking_map,
    };

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&data).unwrap();

    // metadata
    {
        let meta = format!("const DATA_LEN: usize = {};", bytes.len());
        let meta_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("id_map_meta.rs");
        fs::write(&meta_path, meta).unwrap();
    }

    #[cfg(feature = "compress")]
    let bytes = miniz_oxide::deflate::compress_to_vec(&bytes, 9);

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("id_map.bin");
    fs::write(&out_path, bytes).unwrap();
}
