// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[derive(rkyv::Archive, rkyv::Serialize)]
struct Data {
    title_map: BTreeMap<u32, usize>,

    #[cfg(feature = "gamehacking")]
    gamehacking_map: BTreeMap<u32, usize>,

    #[cfg(feature = "ascii-titles")]
    ascii_title_map: BTreeMap<u32, usize>,

    all_titles: Vec<String>,
}

fn make_title_list<'a>(title_maps: &[&'a BTreeMap<u32, Cow<'a, str>>]) -> Vec<String> {
    let mut all_titles = BTreeSet::new();

    for title_map in title_maps {
        for title in title_map.values() {
            all_titles.insert(title.clone().into_owned());
        }
    }

    all_titles.into_iter().collect()
}

fn parse_titles_txt(content: &str) -> BTreeMap<u32, Cow<'_, str>> {
    let mut entries = BTreeMap::new();

    for line in content.lines().skip(1) {
        let (id, title) = line.split_once(" = ").unwrap();
        let id = u32::from_str_radix(id, 36).unwrap();

        entries.insert(id, title.into());
    }

    entries
}

fn make_title_map(
    wiitdb: BTreeMap<u32, Cow<'_, str>>,
    all_titles: &Vec<String>,
) -> BTreeMap<u32, usize> {
    let mut entries = BTreeMap::new();

    for (id, title) in wiitdb {
        let idx = all_titles
            .binary_search_by(|t| t.as_str().cmp(&title))
            .unwrap();
        entries.insert(id, idx);
    }

    entries
}

#[cfg(feature = "ascii-titles")]
fn make_ascii_map<'a>(
    title_map: &BTreeMap<u32, Cow<'a, str>>,
    en_title_map: &'a BTreeMap<u32, Cow<'a, str>>,
) -> BTreeMap<u32, Cow<'a, str>> {
    let mut entries = BTreeMap::new();

    for (id, en_title) in en_title_map {
        let og_title = title_map.get(id).unwrap();
        if og_title.is_ascii() {
            // original title is already ascii, don't add an entry
            // we handle this by falling back to the original title
            continue;
        }

        if en_title.is_ascii() {
            entries.insert(*id, en_title.clone());
        } else {
            let mut ascii_title = en_title.chars().filter(char::is_ascii).collect::<String>();

            let trimmed = ascii_title.trim();
            if ascii_title.len() != trimmed.len() {
                ascii_title = trimmed.to_string();
            }

            assert!(!ascii_title.is_empty());

            entries.insert(*id, ascii_title.into());
        }
    }

    entries
}

#[cfg(feature = "gamehacking")]
fn parse_gamehacking_ids() -> BTreeMap<u32, usize> {
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

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/**");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let titles_txt = fs::read_to_string("assets/wiitdb.txt").unwrap();
    let titles = parse_titles_txt(&titles_txt);

    #[cfg(feature = "ascii-titles")]
    let en_titles_txt = fs::read_to_string("assets/wiitdb-en.txt").unwrap();
    #[cfg(feature = "ascii-titles")]
    let en_titles = parse_titles_txt(&en_titles_txt);
    #[cfg(feature = "ascii-titles")]
    let ascii_titles = make_ascii_map(&titles, &en_titles);

    // a binary searchable vec
    #[cfg(not(feature = "ascii-titles"))]
    let all_titles = make_title_list(&[&titles]);
    #[cfg(feature = "ascii-titles")]
    let all_titles = make_title_list(&[&titles, &ascii_titles]);

    let title_map = make_title_map(titles, &all_titles);

    #[cfg(feature = "gamehacking")]
    let gamehacking_map = parse_gamehacking_ids();

    #[cfg(feature = "ascii-titles")]
    let ascii_title_map = make_title_map(ascii_titles, &all_titles);

    let data = Data {
        title_map,
        #[cfg(feature = "gamehacking")]
        gamehacking_map,
        #[cfg(feature = "ascii-titles")]
        ascii_title_map,
        all_titles,
    };

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&data).unwrap();

    let meta = format!("const DATA_SIZE: usize = {};", bytes.len());

    #[cfg(feature = "compress")]
    let bytes = miniz_oxide::deflate::compress_to_vec(&bytes, 9);

    let out_path = out_dir.join("id_map.bin");
    fs::write(out_path, &bytes).unwrap();

    let meta_out_path = out_dir.join("id_map.rs");
    fs::write(meta_out_path, meta).unwrap();
}
