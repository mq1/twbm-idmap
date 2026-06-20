// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use rkyv::rancor;
use std::{borrow::Cow, collections::BTreeMap, fs, path::PathBuf};

#[cfg(feature = "hashes")]
#[derive(serde::Deserialize)]
struct WiiTdbRom<'a> {
    #[serde(borrow, rename = "@crc", default)]
    crc: Cow<'a, str>,
}

#[derive(serde::Deserialize)]
struct WiiTdbLocale<'a> {
    #[serde(borrow, rename = "@lang")]
    lang: Cow<'a, str>,

    #[serde(borrow)]
    title: Cow<'a, str>,
}

#[derive(serde::Deserialize)]
struct WiiTdbGame<'a> {
    #[serde(borrow)]
    id: Cow<'a, str>,

    #[serde(borrow, rename = "locale", default)]
    locales: Vec<WiiTdbLocale<'a>>,

    #[cfg(feature = "hashes")]
    #[serde(borrow, rename = "rom")]
    roms: Vec<WiiTdbRom<'a>>,
}

#[derive(serde::Deserialize)]
struct WiiTdbDatafile<'a> {
    #[serde(borrow, rename = "game")]
    games: Vec<WiiTdbGame<'a>>,
}

#[derive(rkyv::Archive, rkyv::Serialize)]
struct Game {
    title: String,

    #[cfg(feature = "hashes")]
    crc32s: Vec<u32>,
}

#[derive(rkyv::Archive, rkyv::Serialize)]
struct Data {
    title_map: BTreeMap<u32, Game>,

    #[cfg(feature = "gamehacking")]
    gamehacking_map: BTreeMap<u32, u32>,
}

fn make_title_map(content: &str) -> BTreeMap<u32, Game> {
    let mut entries = BTreeMap::new();

    let datafile = quick_xml::de::from_str::<WiiTdbDatafile>(content).unwrap();
    for game in datafile.games.into_iter().filter(|g| !g.locales.is_empty()) {
        let game_id = u32::from_str_radix(&game.id, 36).unwrap();

        let title = game
            .locales
            .into_iter()
            .find(|l| l.lang == "EN")
            .unwrap()
            .title
            .into_owned();

        #[cfg(feature = "hashes")]
        let crc32s = game
            .roms
            .iter()
            .filter(|r| !r.crc.is_empty())
            .filter_map(|r| u32::from_str_radix(&r.crc, 16).ok())
            .collect();

        let game = Game {
            title,

            #[cfg(feature = "hashes")]
            crc32s,
        };

        entries.insert(game_id, game);
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

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/wiitdb.txt");
    println!("cargo::rerun-if-changed=assets/gamehacking/**");

    let titles_txt = fs::read_to_string("assets/wiitdb.xml").unwrap();
    let title_map = make_title_map(&titles_txt);

    #[cfg(feature = "gamehacking")]
    let gamehacking_map = parse_gamehacking_ids();

    let data = Data {
        title_map,

        #[cfg(feature = "gamehacking")]
        gamehacking_map,
    };

    let bytes = rkyv::to_bytes::<rancor::Error>(&data).unwrap();

    #[cfg(feature = "compress")]
    let meta = format!("const DATA_SIZE: usize = {};", bytes.len());

    #[cfg(feature = "compress")]
    let bytes = miniz_oxide::deflate::compress_to_vec(&bytes, 9);

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let out_path = out_dir.join("id_map.bin");
    fs::write(out_path, bytes).unwrap();

    #[cfg(feature = "compress")]
    {
        let meta_path = out_dir.join("id_map_meta.rs");
        fs::write(meta_path, meta).unwrap();
    }
}
