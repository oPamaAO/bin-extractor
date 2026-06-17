//! Extract structured data from decrypted Albion Online `.bin` XML files.
//!
//! Each function parses a specific XML format and returns typed entries.

use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;

// ── Items ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ItemEntry {
    pub index: u32,
    pub uniquename: String,
}

/// Parse items.xml → list of `{index, uniquename}`.
///
/// Items have no `index` attribute — the numeric protocol ID is their
/// ORDER in the XML (matching items.txt line numbers).
pub fn extract_items(xml: &str) -> Result<Vec<ItemEntry>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut items = Vec::new();
    let mut index: u32 = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                if let Some(un) = get_attr(e, "uniquename") {
                    items.push(ItemEntry { index, uniquename: un });
                    index += 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("items XML parse error: {e}"),
            _ => {}
        }
        buf.clear();
    }
    Ok(items)
}

// ── World Clusters ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ClusterEntry {
    pub id: String,
    pub displayname: String,
    pub cluster_type: String,
    pub biome: String,
}

/// Parse world.xml → list of `{id, displayname, cluster_type, biome}`.
pub fn extract_world(xml: &str) -> Result<Vec<ClusterEntry>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut clusters = Vec::new();
    let mut in_cluster = false;
    let mut current_id = String::new();
    let mut current_display = String::new();
    let mut current_type = String::new();
    let mut current_biome = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                if e.name().as_ref() == b"cluster" {
                    current_id = get_attr(e, "id").unwrap_or_default();
                    current_display = get_attr(e, "displayname").unwrap_or_default();
                    current_type = get_attr(e, "type").unwrap_or_default();
                    current_biome = String::new();
                    in_cluster = true;
                } else if e.name().as_ref() == b"biome" && in_cluster {
                    current_biome = get_attr(e, "type").unwrap_or_default();
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"cluster" && in_cluster {
                    clusters.push(ClusterEntry {
                        id: std::mem::take(&mut current_id),
                        displayname: std::mem::take(&mut current_display),
                        cluster_type: std::mem::take(&mut current_type),
                        biome: std::mem::take(&mut current_biome),
                    });
                    in_cluster = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("world XML parse error: {e}"),
            _ => {}
        }
        buf.clear();
    }
    Ok(clusters)
}

// ── Mobs ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MobEntry {
    pub index: u32,
    pub uniquename: String,
}

/// Parse mobs.xml → list of `{index, uniquename}`.
pub fn extract_mobs(xml: &str) -> Result<Vec<MobEntry>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut mobs = Vec::new();
    let mut index: u32 = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                if e.name().as_ref() == b"Mob" {
                    if let Some(un) = get_attr(e, "uniquename") {
                        mobs.push(MobEntry { index, uniquename: un });
                        index += 1;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("mobs XML parse error: {e}"),
            _ => {}
        }
        buf.clear();
    }
    Ok(mobs)
}

// ── Spells ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SpellEntry {
    pub index: u32,
    pub name: String,
}

/// Parse spells.bin → list of `{index, name}`.
///
/// XML tags: `<activespell>`, `<passivespell>`, `<togglespell>`
pub fn extract_spells(xml: &str) -> Result<Vec<SpellEntry>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut spells = Vec::new();
    let mut index: u32 = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let is_spell = {
                    let qn = e.name();
                    let tag = qn.as_ref();
                    tag == b"activespell" || tag == b"passivespell" || tag == b"togglespell"
                };
                if is_spell {
                    let name = get_attr(e, "uniquename")
                        .or_else(|| get_attr(e, "name"))
                        .unwrap_or_default();
                    spells.push(SpellEntry { index, name });
                    index += 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("spells XML parse error: {e}"),
            _ => {}
        }
        buf.clear();
    }
    Ok(spells)
}

// ── Buildings ────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BuildingEntry {
    pub index: u32,
    pub uniquename: String,
    pub displayname: String,
}

/// Parse buildings.bin → list of `{index, uniquename, displayname}`.
///
/// Building elements use type-specific tags like `<simplebehaviourbuilding>`,
/// `<craftbuilding>`, `<bankbuilding>`, etc. All have `uniquename`.
pub fn extract_buildings(xml: &str) -> Result<Vec<BuildingEntry>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut buildings = Vec::new();
    let mut index: u32 = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let qn = e.name();
                let tag = qn.as_ref();
                let tag_ends_building = tag.len() > 8 && tag.ends_with(b"building");
                if tag_ends_building || tag == b"labourer" || tag == b"fortificationtype" || tag == b"durabilitythresholddefinition" {
                    let uniquename = get_attr(e, "uniquename").unwrap_or_default();
                    let displayname = get_attr(e, "displayname").unwrap_or_default();
                    buildings.push(BuildingEntry { index, uniquename, displayname });
                    index += 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("buildings XML parse error: {e}"),
            _ => {}
        }
        buf.clear();
    }
    Ok(buildings)
}

// ── Localization (TMX) ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LocalizationEntry {
    pub key: String,
    pub value: String,
    pub language: String,
}

/// Parse localization.bin (TMX format) → list of translations.
pub fn extract_localization(xml: &str) -> Result<Vec<LocalizationEntry>> {
    use std::collections::HashMap;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut results = Vec::new();

    enum State {
        Outside,
        InTu,
        InTuv { lang: String },
    }
    let mut state = State::Outside;
    let mut seg_text = String::new();
    let mut tu_map: HashMap<String, String> = HashMap::new();
    let mut in_seg = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"tu" {
                    tu_map.clear();
                    state = State::InTu;
                } else if e.name().as_ref() == b"tuv" {
                    let lang = get_attr(e, "xml:lang")
                        .or_else(|| get_attr(e, "lang"))
                        .unwrap_or_default();
                    state = State::InTuv { lang };
                } else if e.name().as_ref() == b"seg" {
                    seg_text.clear();
                    in_seg = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_seg {
                    if let Ok(text) = e.unescape() {
                        seg_text.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"seg" {
                    in_seg = false;
                } else if e.name().as_ref() == b"tuv" {
                    if let State::InTuv { lang } = &state {
                        let lang_val = if lang.is_empty() { "en-US".to_string() } else { lang.clone() };
                        tu_map.insert(lang_val, seg_text.clone());
                    }
                    state = State::InTu;
                } else if e.name().as_ref() == b"tu" {
                    for (lang, value) in &tu_map {
                        results.push(LocalizationEntry {
                            key: String::new(),
                            value: value.clone(),
                            language: lang.clone(),
                        });
                    }
                    state = State::Outside;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("localization XML parse error: {e}"),
            _ => {}
        }
        buf.clear();
    }

    Ok(results)
}

// ── GameData overview (metadata) ───────────────────────────────────

#[derive(Debug, Serialize)]
pub struct GameDataEntry {
    pub key: String,
    pub value: String,
}

/// Parse gamedata.bin (simple key-value XML).
pub fn extract_gamedata(xml: &str) -> Result<Vec<GameDataEntry>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut entries = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                if let Some(key) = get_attr(e, "name")
                    .or_else(|| get_attr(e, "key"))
                {
                    let value = get_attr(e, "value").unwrap_or_default();
                    entries.push(GameDataEntry { key, value });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("gamedata XML parse error: {e}"),
            _ => {}
        }
        buf.clear();
    }
    Ok(entries)
}

// ── Helper ───────────────────────────────────────────────────────────

fn get_attr(e: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
}
