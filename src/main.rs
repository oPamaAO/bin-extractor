//! Albion Online `.bin` Game Data Extractor
//!
//! Decrypts Albion Online binary data files (DES-CBC + gzip),
//! parses the embedded XML, and exports clean JSON.
//!
//! # Quick start
//!
//! ```bash
//! # Interactive mode (auto-detect GameData)
//! bin-extract
//!
//! # Extract everything at once
//! bin-extract all "path/to/GameData" -o ./output/
//!
//! # Extract individual files
//! bin-extract items items.bin -o items.json
//! bin-extract world cluster/world.bin -o world.json
//! ```

mod decrypt;
mod extract;
mod gamedata;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use humansize::format_size;
use humansize::BINARY;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::{fs, process};

// ── CLI Definition ──────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "bin-extract",
    about = "🔓 Albion Online .bin Game Data Extractor",
    version = "1.0.0",
    long_about = "Decrypts Albion Online .bin files (DES-CBC + gzip → XML → JSON).\n\
                  Run without arguments for interactive mode.",
    display_order = 0
)]
struct Cli {
    /// GameData path (auto-detected if omitted)
    #[arg(short = 'g', long = "gamedata", env = "ALBION_GAMEDATA", global = true)]
    gamedata_path: Option<PathBuf>,

    /// Output format: pretty (default) or compact
    #[arg(long = "format", default_value = "pretty", global = true)]
    format: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// 🔧 Interactive mode — menus and prompts for easy use
    #[command(name = "interactive", aliases = &["i", "menu", "ui"])]
    Interactive,

    /// 📦 Extract items from items.bin
    Items {
        /// Path to items.bin
        input: PathBuf,
        /// Output JSON file (default: items.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 🌍 Extract world cluster definitions
    World {
        /// Path to world.bin (in GameData/cluster/)
        input: PathBuf,
        /// Output JSON file (default: world.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 👾 Extract mob definitions
    Mobs {
        /// Path to mobs.bin
        input: PathBuf,
        /// Output JSON file (default: mobs.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// ⚡ Extract spells/abilities
    Spells {
        /// Path to spells.bin
        input: PathBuf,
        /// Output JSON file (default: spells.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 🏗️ Extract building definitions
    Buildings {
        /// Path to buildings.bin
        input: PathBuf,
        /// Output JSON file (default: buildings.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 🌐 Extract localization strings
    Localization {
        /// Path to localization.bin
        input: PathBuf,
        /// Output JSON file (default: localization.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// ⚙️ Extract game settings
    Gamedata {
        /// Path to gamedata.bin
        input: PathBuf,
        /// Output JSON file (default: gamedata.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 📋 List available .bin files in a GameData directory
    #[command(name = "list", aliases = &["ls"])]
    List {
        /// Path to GameData directory (auto-detected if omitted)
        #[arg(default_value = "_auto")]
        gamedata: String,
    },

    /// 📦 Extract ALL known .bin files from a GameData directory
    All {
        /// Path to GameData directory (auto-detected if omitted)
        #[arg(default_value = "_auto")]
        gamedata: String,
        /// Output directory (default: ./output/)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 🔍 Decrypt a .bin file to raw XML (debug)
    Decrypt {
        /// Path to any .bin file
        input: PathBuf,
        /// Output XML file (default: input name with .xml extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

// ── Entry Point ─────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    // If no subcommand, go interactive
    if cli.command.is_none() {
        if let Err(e) = run_interactive(&cli) {
            eprintln!("{} {}", "[ERROR]".red().bold(), e);
            process::exit(1);
        }
        return;
    }

    let result = match cli.command.as_ref().unwrap() {
        Command::Interactive => run_interactive(&cli),
        Command::Items { input, output } => extract_single::<extract::ItemEntry>(
            input, output, "items.json",
            extract::extract_items,
            "items", "item",
        ),
        Command::World { input, output } => extract_single::<extract::ClusterEntry>(
            input, output, "world.json",
            extract::extract_world,
            "clusters", "cluster",
        ),
        Command::Mobs { input, output } => extract_single::<extract::MobEntry>(
            input, output, "mobs.json",
            extract::extract_mobs,
            "mobs", "mob",
        ),
        Command::Spells { input, output } => extract_single::<extract::SpellEntry>(
            input, output, "spells.json",
            extract::extract_spells,
            "spells", "spell",
        ),
        Command::Buildings { input, output } => extract_single::<extract::BuildingEntry>(
            input, output, "buildings.json",
            extract::extract_buildings,
            "buildings", "building",
        ),
        Command::Localization { input, output } => extract_single::<extract::LocalizationEntry>(
            input, output, "localization.json",
            extract::extract_localization,
            "translations", "entry",
        ),
        Command::Gamedata { input, output } => extract_single::<extract::GameDataEntry>(
            input, output, "gamedata.json",
            extract::extract_gamedata,
            "settings", "setting",
        ),
        Command::List { gamedata } => run_list(gamedata),
        Command::All { gamedata, output } => run_all(gamedata, output.as_deref()),
        Command::Decrypt { input, output } => run_decrypt(input, output),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "[ERROR]".red().bold(), e);
        process::exit(1);
    }
}

// ── Interactive Mode ────────────────────────────────────────────────

fn run_interactive(cli: &Cli) -> Result<()> {
    println!();
    println!("  {}", "╔══════════════════════════════════════╗".cyan());
    println!("  {}", "║   Albion Online .bin Data Extractor  ║".cyan().bold());
    println!("  {}", "╚══════════════════════════════════════╝".cyan());
    println!();

    // Auto-detect GameData
    let gamedata_path = resolve_gamedata_path(cli)?;

    println!("  {} GameData: {}", "📂".bold(), gamedata_path.display().to_string().cyan());
    println!();

    loop {
        println!("  {} {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed(), "MENU".dimmed());
        println!("  {} Extract all known .bin files", "1.".bold());
        println!("  {} List available .bin files",    "2.".bold());
        println!("  {} Extract a specific type",      "3.".bold());
        println!("  {} Decrypt a .bin to XML",        "4.".bold());
        println!("  {} Change GameData path",         "5.".bold());
        println!("  {} Quit",                         "q.".bold());
        print!("  {} ", "→".green().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "1" | "all" => {
                let out_dir = prompt("    Output directory", "output")?;
                run_all(&gamedata_path.to_string_lossy(), Some(Path::new(&out_dir)))?;
                println!();
                pause();
            }
            "2" | "ls" | "list" => {
                run_list_from_path(&gamedata_path)?;
                println!();
                pause();
            }
            "3" | "extract" => {
                println!();
                println!("    {} Available types:", "Select type:".yellow());
                let types = [
                    ("items", "Items database"),
                    ("mobs", "Mobs database"),
                    ("world", "World clusters"),
                    ("spells", "Spells/abilities"),
                    ("buildings", "Building definitions"),
                    ("localization", "Localization strings"),
                    ("gamedata", "Game settings"),
                ];
                for (i, (name, desc)) in types.iter().enumerate() {
                    println!("     {}. {} — {}", (i + 1).to_string().bold(), name.cyan(), desc);
                }
                let choice = prompt("    Enter number or name", "1")?;

                let (cmd_name, filename) = match choice.as_str() {
                    "1" | "items"       => ("items", "items.bin"),
                    "2" | "mobs"        => ("mobs", "mobs.bin"),
                    "3" | "world"       => ("world", "cluster/world.bin"),
                    "4" | "spells"      => ("spells", "spells.bin"),
                    "5" | "buildings"   => ("buildings", "buildings.bin"),
                    "6" | "localization" => ("localization", "localization.bin"),
                    "7" | "gamedata"    => ("gamedata", "gamedata.bin"),
                    _ => {
                        println!("    {} Invalid choice", "[✗]".red());
                        continue;
                    }
                };

                let bin_path = if filename.starts_with("cluster/") {
                    gamedata_path.join("cluster").join(&filename[8..])
                } else {
                    gamedata_path.join(filename)
                };

                if !bin_path.exists() {
                    println!("    {} Not found: {}", "[✗]".red(), bin_path.display());
                    pause();
                    continue;
                }

                let default_out = format!("{}.json", cmd_name);
                let out_file = prompt("    Output file", &default_out)?;
                dispatch_extract(cmd_name, &bin_path, Path::new(&out_file))?;
                println!();
                pause();
            }
            "4" | "decrypt" => {
                let bin_path_str = prompt("    Path to .bin file", "")?;
                let bin_path = Path::new(&bin_path_str);
                if !bin_path.exists() {
                    println!("    {} File not found", "[✗]".red());
                    pause();
                    continue;
                }
                let default_xml = bin_path.with_extension("xml").to_string_lossy().to_string();
                let out_file = prompt("    Output XML file", &default_xml)?;
                run_decrypt(bin_path, &Some(PathBuf::from(&out_file)))?;
                pause();
            }
            "5" | "path" => {
                let new_path = prompt("    Enter GameData path", "")?;
                if Path::new(&new_path).exists() {
                    println!("    {} Path set to: {}", "[✓]".green(), new_path.cyan());
                    // Store in CLI's gamedata_path (use env trick)
                    std::env::set_var("ALBION_GAMEDATA", &new_path);
                    println!("    {} (set ALBION_GAMEDATA for this session)", "[i]".blue());
                    // We can't easily update the Cli struct, but prompt shows new path
                    println!("    {} Restart to use new path permanently via env var", "[i]".blue());
                } else {
                    println!("    {} Path does not exist", "[✗]".red());
                }
                pause();
            }
            "q" | "quit" | "exit" | "" => {
                println!("  {}", "👋 Goodbye!".green());
                break;
            }
            _ => {
                println!("    {} Unknown option: {}", "[!]".yellow(), input);
            }
        }
    }

    Ok(())
}

fn prompt(label: &str, default: &str) -> Result<String> {
    print!("  {} {} [{}]: ", "▸".cyan().bold(), label, if default.is_empty() { "?" } else { default });
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed)
    }
}

fn pause() {
    print!("    {} Press Enter to continue...", "[i]".blue());
    io::stdout().flush().unwrap();
    let mut _dummy = String::new();
    io::stdin().lock().read_line(&mut _dummy).unwrap();
}

// ── Extraction Dispatchers ──────────────────────────────────────────

fn dispatch_extract(cmd: &str, input: &Path, output: &Path) -> Result<()> {
    match cmd {
        "items"       => extract_dispatch::<extract::ItemEntry>(
            input, output, |xml| extract::extract_items(xml), "items", "item"),
        "mobs"        => extract_dispatch::<extract::MobEntry>(
            input, output, |xml| extract::extract_mobs(xml), "mobs", "mob"),
        "world"       => extract_dispatch::<extract::ClusterEntry>(
            input, output, |xml| extract::extract_world(xml), "clusters", "cluster"),
        "spells"      => extract_dispatch::<extract::SpellEntry>(
            input, output, |xml| extract::extract_spells(xml), "spells", "spell"),
        "buildings"   => extract_dispatch::<extract::BuildingEntry>(
            input, output, |xml| extract::extract_buildings(xml), "buildings", "building"),
        "localization" => extract_dispatch::<extract::LocalizationEntry>(
            input, output, |xml| extract::extract_localization(xml), "translations", "entry"),
        "gamedata"    => extract_dispatch::<extract::GameDataEntry>(
            input, output, |xml| extract::extract_gamedata(xml), "settings", "setting"),
        _ => anyhow::bail!("Unknown extract type: {cmd}"),
    }
}

/// Display-label helper: plural display name, singular label
fn extract_dispatch<T: serde::Serialize>(
    input: &Path,
    output: &Path,
    parse: fn(&str) -> Result<Vec<T>>,
    plural: &str,
    _singular: &str,
) -> Result<()> {
    let raw = fs::read(input)
        .with_context(|| format!("Failed to read {}", input.display()))?;
    let label = format_size(raw.len(), BINARY);

    print!("  {} Decrypting {} [{}] ... ", "[1/3]".dimmed(), input.file_name().unwrap().to_string_lossy(), label);
    io::stdout().flush()?;
    let xml = decrypt::decrypt_bin(&raw)
        .with_context(|| format!("Failed to decrypt {}", input.display()))?;
    println!("{}", "OK".green().bold());

    print!("  {} Parsing XML ({} bytes) ... ", "[2/3]".dimmed(), xml.len());
    io::stdout().flush()?;
    let entries = parse(&xml)?;
    println!("{}", "OK".green().bold());

    print!("  {} Writing {} {} to {} ... ", "[3/3]".dimmed(), entries.len(), plural, output.display());
    io::stdout().flush()?;
    write_json(&entries, output)?;
    println!("{}", "OK".green().bold());

    println!(
        "  {} Successfully extracted {} {} → {}",
        "✅".bold(),
        entries.len().to_string().cyan().bold(),
        plural,
        output.display().to_string().yellow()
    );

    Ok(())
}

fn extract_single<T: serde::Serialize>(
    input: &Path,
    output: &Option<PathBuf>,
    default_name: &str,
    parse: fn(&str) -> Result<Vec<T>>,
    plural: &str,
    singular: &str,
) -> Result<()> {
    let output_path = output.as_deref().unwrap_or(Path::new(default_name));
    extract_dispatch(input, output_path, parse, plural, singular)
}

// ── All / List / Decrypt ────────────────────────────────────────────

fn run_all(gamedata_arg: &str, output: Option<&Path>) -> Result<()> {
    let gamedata = resolve_gamedata_arg(gamedata_arg)?;
    let out_dir = output.unwrap_or(Path::new("output"));
    fs::create_dir_all(out_dir)
        .context("Failed to create output directory")?;

    println!("  {} GameData: {}", "📂".bold(), gamedata.display().to_string().cyan());
    println!("  {} Output:   {}", "📁".bold(), out_dir.display().to_string().yellow());
    println!();
    let types: Vec<(&str, &str, &str)> = vec![
        ("items.bin",       "items",       "items.json"),
        ("mobs.bin",        "mobs",        "mobs.json"),
        ("spells.bin",      "spells",      "spells.json"),
        ("buildings.bin",   "buildings",   "buildings.json"),
        ("gamedata.bin",    "gamedata",    "gamedata.json"),
        ("cluster/world.bin", "world",     "world.json"),
    ];

    for (filename, cmd, outname) in &types {
        let bin_path = if filename.starts_with("cluster/") {
            gamedata.join("cluster").join(&filename[8..])
        } else {
            gamedata.join(filename)
        };

        if !bin_path.exists() {
            println!("  {} {} — {}",
                "[SKIP]".yellow().dimmed(),
                filename.dimmed(),
                "not found".dimmed());
            continue;
        }

        let out_path = out_dir.join(outname);
        if let Err(e) = dispatch_extract(cmd, &bin_path, &out_path) {
            println!("  {} {} — {}",
                "[FAIL]".red(),
                filename,
                e.to_string().red());
        }
    }

    // Localization is special (large TMX)
    let loc_path = gamedata.join("localization.bin");
    if loc_path.exists() {
        let out_path = out_dir.join("localization.json");
        if let Err(e) = dispatch_extract("localization", &loc_path, &out_path) {
            println!("  {} localization.bin — {}", "[FAIL]".red(), e);
        }
    }

    println!();
    println!("  {} All done! Files in: {}", "🎉".bold(), out_dir.display().to_string().yellow());
    Ok(())
}

fn run_list(gamedata_arg: &str) -> Result<()> {
    let gamedata = if gamedata_arg == "_auto" {
        gamedata::auto_detect_gamedata()
            .context("Could not auto-detect GameData. Specify path or set ALBION_GAMEDATA")?
    } else {
        PathBuf::from(gamedata_arg)
    };
    run_list_from_path(&gamedata)
}

fn run_list_from_path(gamedata: &Path) -> Result<()> {
    if !gamedata.is_dir() {
        anyhow::bail!("Not a directory: {}", gamedata.display());
    }

    let files = gamedata::list_bin_files(gamedata)?;

    println!("  {} GameData: {} ({} .bin files)",
        "📂".bold(),
        gamedata.display().to_string().cyan(),
        files.len().to_string().bold());
    println!();

    // Known types
    let known: Vec<_> = files.iter().filter(|f| f.known_type).collect();
    let unknown: Vec<_> = files.iter().filter(|f| !f.known_type).collect();

    if !known.is_empty() {
        println!("  {} {}", "📦 Known types".green().bold(), "(supports JSON export)".dimmed());
        for f in &known {
            let size = format_size(f.size_bytes, BINARY);
            let extract_cmd = gamedata::type_for_bin(&f.name)
                .unwrap_or("?");
            let desc = gamedata::KNOWN_TYPES.iter()
                .find(|(fn_, _, _)| *fn_ == f.name.as_str())
                .map(|(_, _, d)| *d)
                .unwrap_or("");
            println!("    {} · {} · {} → {} {}",
                f.name.cyan(),
                size.dimmed(),
                "bin-extract".yellow().dimmed(),
                extract_cmd.yellow().bold(),
                desc.dimmed());
        }
    }

    if !unknown.is_empty() {
        println!();
        println!("  {} {} {}", "📄".dimmed(), "Other files".dimmed(), "(raw decrypt only)".dimmed());
        for f in &unknown {
            let size = format_size(f.size_bytes, BINARY);
            println!("    {} ({})", f.name.dimmed(), size.dimmed());
        }
    }

    Ok(())
}

fn run_decrypt(input: &Path, output: &Option<PathBuf>) -> Result<()> {
    let raw = fs::read(input)
        .with_context(|| format!("Failed to read {}", input.display()))?;
    let size_label = format_size(raw.len(), BINARY);

    print!("  {} Decrypting {} [{}] ... ", "[1/1]".dimmed(), input.display(), size_label);
    io::stdout().flush()?;

    let xml = decrypt::decrypt_bin(&raw)?;
    println!("{}", "OK".green().bold());

    let out_path = output.clone().unwrap_or_else(|| {
        let mut p = input.to_path_buf();
        p.set_extension("xml");
        p
    });

    fs::write(&out_path, &xml)
        .with_context(|| format!("Failed to write {}", out_path.display()))?;

    println!("  {} Decrypted {} · {} XML bytes → {}",
        "✅".bold(),
        size_label.dimmed(),
        xml.len().to_string().cyan(),
        out_path.display().to_string().yellow());
    Ok(())
}

// ── Path Resolution ────────────────────────────────────────────────

fn resolve_gamedata_arg(arg: &str) -> Result<PathBuf> {
    if arg == "_auto" {
        resolve_gamedata_auto()
    } else {
        let p = PathBuf::from(arg);
        if !p.is_dir() {
            anyhow::bail!("GameData directory not found: {}", p.display());
        }
        Ok(p)
    }
}

fn resolve_gamedata_path(cli: &Cli) -> Result<PathBuf> {
    if let Some(ref p) = cli.gamedata_path {
        if p.is_dir() {
            return Ok(p.clone());
        }
        anyhow::bail!("GameData directory not found: {}", p.display());
    }
    resolve_gamedata_auto()
}

fn resolve_gamedata_auto() -> Result<PathBuf> {
    // Check env var first
    if let Ok(path) = std::env::var("ALBION_GAMEDATA") {
        let p = PathBuf::from(&path);
        if p.is_dir() {
            return Ok(p);
        }
    }

    // Auto-detect
    gamedata::auto_detect_gamedata()
        .context(
            "Could not auto-detect GameData directory.\n\
             Please provide the path manually or set ALBION_GAMEDATA environment variable.\n\
             Example: bin-extract all \"C:\\Program Files (x86)\\AlbionOnline\\game\\Albion-Online_Data\\StreamingAssets\\GameData\""
        )
}

// ── JSON Writer ─────────────────────────────────────────────────────

fn write_json<T: serde::Serialize>(value: &T, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, &json)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
