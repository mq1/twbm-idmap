// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

fn parse_titles_txt(content: &str) -> Vec<(u32, &str)> {
    let mut entries = Vec::with_capacity(16_384);

    for line in content.lines().skip(1) {
        let (id, title) = line.split_once(" = ").unwrap();
        let id = u32::from_str_radix(id, 36).unwrap();

        entries.push((id, title));
    }

    entries
}

#[cfg(feature = "ascii-titles")]
fn make_ascii_map<'a>(
    title_map: &[(u32, &str)],
    en_title_map: &[(u32, &'a str)],
) -> Vec<(u32, std::borrow::Cow<'a, str>)> {
    let mut entries = Vec::with_capacity(4096);

    for ((id, title), (en_id, en_title)) in
        title_map.iter().copied().zip(en_title_map.iter().copied())
    {
        assert_eq!(id, en_id);

        if title.is_ascii() {
            // original title is already ascii, don't add an entry
            // we handle this by falling back to the original title
            continue;
        }

        if en_title.is_ascii() {
            entries.push((en_id, en_title.into()));
        } else {
            let mut ascii_title = en_title
                .chars()
                .filter(char::is_ascii)
                .skip_while(char::is_ascii_whitespace)
                .collect::<String>();

            ascii_title.truncate(ascii_title.trim_end().len());

            assert!(!ascii_title.is_empty());
            entries.push((en_id, ascii_title.into()));
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

    let titles_txt = fs::read_to_string("assets/wiitdb.txt").unwrap();
    let mut titles = parse_titles_txt(&titles_txt);

    #[cfg(feature = "ascii-titles")]
    let en_titles_txt = fs::read_to_string("assets/wiitdb-en.txt").unwrap();
    #[cfg(feature = "ascii-titles")]
    let en_titles = parse_titles_txt(&en_titles_txt);
    #[cfg(feature = "ascii-titles")]
    let mut ascii_titles = make_ascii_map(&titles, &en_titles);

    titles.sort_unstable_by_key(|(id, _)| *id);
    titles.dedup_by_key(|(id, _)| *id);

    #[cfg(feature = "ascii-titles")]
    ascii_titles.sort_unstable_by_key(|(id, _)| *id);
    #[cfg(feature = "ascii-titles")]
    ascii_titles.dedup_by_key(|(id, _)| *id);

    #[cfg(feature = "gamehacking")]
    let gamehacking_map = parse_gamehacking_ids();

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("id_map.rs");
    let out_file = File::create(&out_path).unwrap();
    let mut out = BufWriter::new(out_file);

    // title map
    {
        write!(
            &mut out,
            "#[cfg(not(clippy))]\npub const TITLE_MAP: &[(u32,&str);{}] = &[",
            titles.len()
        )
        .unwrap();

        for (game_id, game_title) in &titles {
            write!(&mut out, "({game_id},r#\"{game_title}\"#),").unwrap();
        }

        out.write_all(b"];\n#[cfg(clippy)]\npub const TITLE_MAP: &[(u32,&str);0] = &[];\n")
            .unwrap();
    }

    // gamehacking map
    #[cfg(feature = "gamehacking")]
    {
        write!(
            &mut out,
            "#[cfg(not(clippy))]\npub const GAMEHACKING_MAP: &[(u32,u32);{}] = &[",
            gamehacking_map.len()
        )
        .unwrap();

        for (game_id, ghid) in gamehacking_map {
            write!(&mut out, "({game_id},{ghid}),").unwrap();
        }

        out.write_all(b"];\n#[cfg(clippy)]\npub const GAMEHACKING_MAP: &[(u32,u32);0] = &[];\n")
            .unwrap();
    }

    // ascii title map
    #[cfg(feature = "ascii-titles")]
    {
        write!(
            &mut out,
            "#[cfg(not(clippy))]\npub const ASCII_TITLE_MAP: &[(u32,&str);{}] = &[",
            ascii_titles.len()
        )
        .unwrap();

        for (game_id, game_title) in ascii_titles {
            write!(&mut out, "({game_id},r#\"{game_title}\"#),").unwrap();
        }

        out.write_all(b"];\n#[cfg(clippy)]\npub const ASCII_TITLE_MAP: &[(u32,&str);0] = &[];\n")
            .unwrap();
    }
}
