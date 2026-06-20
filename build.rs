// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{borrow::Cow, collections::BTreeMap, fmt::Write, fs, path::PathBuf};

#[cfg(feature = "crc32-hashes")]
#[derive(serde::Deserialize)]
struct WiiTdbRom<'a> {
    #[serde(borrow, rename = "@crc")]
    crc32: Option<Cow<'a, str>>,
}

#[derive(serde::Deserialize)]
struct WiiTdbLocale<'a> {
    #[serde(borrow, rename = "@lang")]
    lang: &'a str,

    #[serde(borrow)]
    title: Cow<'a, str>,
}

#[derive(serde::Deserialize)]
struct WiiTdbGame<'a> {
    #[serde(borrow)]
    id: &'a str,

    #[serde(borrow, rename = "locale", default)]
    locales: Vec<WiiTdbLocale<'a>>,

    #[cfg(feature = "crc32-hashes")]
    #[serde(borrow, rename = "rom")]
    roms: Vec<WiiTdbRom<'a>>,
}

#[derive(serde::Deserialize)]
struct WiiTdbDatafile<'a> {
    #[serde(borrow, rename = "game")]
    games: Vec<WiiTdbGame<'a>>,
}

fn make_title_map<'a>(datafile: &'a WiiTdbDatafile<'a>) -> BTreeMap<u32, &'a str> {
    let mut entries = BTreeMap::new();

    for game in &datafile.games {
        let Some(en_locale) = game.locales.iter().find(|l| l.lang == "EN") else {
            continue;
        };

        if en_locale.title.is_empty() || game.id.is_empty() {
            continue;
        }

        let game_id = u32::from_str_radix(game.id, 36).unwrap();

        entries.insert(game_id, en_locale.title.as_ref());
    }

    entries
}

#[cfg(feature = "crc32-hashes")]
fn make_hash_map(datafile: &WiiTdbDatafile) -> BTreeMap<u32, u32> {
    let mut entries = BTreeMap::new();

    for game in &datafile.games {
        if game.id.is_empty() {
            continue;
        }

        let game_id = u32::from_str_radix(game.id, 36).unwrap();

        for rom in &game.roms {
            if let Some(crc32) = rom.crc32.as_deref()
                && !crc32.is_empty()
                && let Ok(crc32) = u32::from_str_radix(crc32, 16)
            {
                entries.insert(crc32, game_id);
            }
        }
    }

    entries
}

#[cfg(feature = "gamehacking")]
fn make_ghid_map() -> BTreeMap<u32, u32> {
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
    println!("cargo::rerun-if-changed=assets/wiitdb.txt");
    println!("cargo::rerun-if-changed=assets/gamehacking/**");

    let target_endian = std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();
    let encode_u32 = |value: u32| match target_endian.as_str() {
        "big" => value.to_be_bytes(),
        "little" => value.to_le_bytes(),
        _ => unreachable!(),
    };

    let wiitdb = fs::read_to_string("assets/wiitdb.xml").unwrap();
    let datafile = quick_xml::de::from_str::<WiiTdbDatafile>(&wiitdb).unwrap();

    #[cfg(feature = "gamehacking")]
    let ghid_map = make_ghid_map();

    let mut bytes = Vec::with_capacity(1 << 20); // 1 MiB
    let mut meta = String::new();

    let title_map = make_title_map(&datafile);
    {
        // title map game ids
        for id in title_map.keys() {
            let id_slice = encode_u32(*id);
            bytes.extend_from_slice(&id_slice);
        }

        // title map title offsets
        let mut cursor = 0u32;
        for title in title_map.values() {
            let title_offset_slice = encode_u32(cursor);
            bytes.extend_from_slice(&title_offset_slice);

            let len = u32::try_from(title.len()).unwrap();
            cursor = cursor.checked_add(len).unwrap();
        }

        // titles end marker
        let titles_end = encode_u32(cursor);
        bytes.extend_from_slice(&titles_end);

        write!(&mut meta, "const TITLE_COUNT: usize = {};", title_map.len()).unwrap();
        write!(&mut meta, "const TITLES_LEN: usize = {};", cursor).unwrap();
    }

    #[cfg(feature = "crc32-hashes")]
    {
        let hash_map = make_hash_map(&datafile);

        // hash map crc32s
        for hash in hash_map.keys() {
            let crc32s_slice = encode_u32(*hash);
            bytes.extend_from_slice(&crc32s_slice);
        }

        // hash map game ids
        for id in hash_map.values() {
            let id_slice = encode_u32(*id);
            bytes.extend_from_slice(&id_slice);
        }

        write!(&mut meta, "const HASH_COUNT: usize = {};", hash_map.len()).unwrap();
    }

    #[cfg(feature = "gamehacking")]
    {
        // gamehacking game ids
        for id in ghid_map.keys() {
            let id_slice = encode_u32(*id);
            bytes.extend_from_slice(&id_slice);
        }

        // gamehacking ghids
        for ghid in ghid_map.values() {
            let ghid_slice = encode_u32(*ghid);
            bytes.extend_from_slice(&ghid_slice);
        }

        write!(&mut meta, "const GHID_COUNT: usize = {};", ghid_map.len()).unwrap();
    }

    // write titles
    for title in title_map.into_values() {
        bytes.extend_from_slice(title.as_bytes());
    }

    #[cfg(feature = "compress")]
    let bytes = miniz_oxide::deflate::compress_to_vec(&bytes, 9);

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let out_path = out_dir.join("id_map.bin");
    fs::write(out_path, bytes).unwrap();

    let meta_path = out_dir.join("id_map_meta.rs");
    fs::write(meta_path, meta).unwrap();
}
