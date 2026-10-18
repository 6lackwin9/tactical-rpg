use aegis_core::{IVec2, Variant};
use aegis_vfs::{CatalogStore, Vfs};
use aegis_vm::{BattleOutcome, CombatGrid, CombatSession, Combatant, Team};
use std::env;
use std::io::{self, BufRead, Write};
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check for help
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    println!("==================================================================");
    println!("===       PROJECT AEGIS - NATIVE ENGINE RUNTIME (v0.1)         ===");
    println!("===          Starting Game Instance on Custom Engine           ===");
    println!("==================================================================");

    let start_time = Instant::now();

    // 1. Virtual File System & Catalog Ingestion
    let vfs = Vfs::discover();
    let mut catalog = CatalogStore::new(vfs.clone());

    print!("[Engine] Discovering game root and loading catalogs... ");
    let loaded = catalog.load_all_standard()?;
    println!(
        "OK! ({} classes, {} skills, {} items, {} encounters, {} units)",
        loaded.get("classes").unwrap_or(&0),
        loaded.get("skills").unwrap_or(&0),
        loaded.get("items").unwrap_or(&0),
        loaded.get("encounters").unwrap_or(&0),
        loaded.get("units").unwrap_or(&0),
    );

    // Check for --load flag
    if let Some(pos) = args.iter().position(|a| a == "--load") {
        let save_path = args.get(pos + 1).map(|s| s.as_str()).unwrap_or("user://saves/slot_0.json");
        return load_and_inspect_save(&vfs, save_path);
    }

    // 2. Encounter Configuration
    let encounter_id = get_arg_value(&args, "--encounter").unwrap_or_else(|| "goblin_skirmish".to_string());
    let rng_seed: u64 = get_arg_value(&args, "--seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1337);
    let interactive = args.iter().any(|a| a == "--interactive" || a == "-i");

    println!("[Engine] Initializing Game World: 'Ashlands' | Seed: {}", rng_seed);

    let (encounter_name, party_ids, enemy_ids, max_rounds) = load_encounter_meta(&catalog, &encounter_id);
    println!("[Engine] Selected Encounter: '{}' ({})", encounter_name, encounter_id);

    // 3. Spawning Combatants
    let party_units: Vec<Combatant> = party_ids
        .iter()
        .map(|id| resolve_unit(id, Team::Allies, &catalog))
        .collect();

    let enemy_units: Vec<Combatant> = enemy_ids
        .iter()
        .map(|id| resolve_unit(id, Team::Enemies, &catalog))
        .collect();

    println!(
        "[Engine] Mounted Battlefield: {} Allies vs {} Enemies",
        party_units.len(),
        enemy_units.len()
    );

    let mut session = CombatSession::new(party_units, enemy_units, rng_seed);
    session.max_rounds = max_rounds;

    // 4. Attach Engine Observer Signals
    session.on_log_emitted.connect(|signal_args| {
        if let Some(msg) = signal_args.first() {
            println!("  [CombatLog] {}", msg.as_str());
        }
    });

    session.on_combatant_defeated.connect(|signal_args| {
        if let Some(name) = signal_args.first() {
            println!("  *** [SIGNAL: DEFEAT] {} was felled in battle! ***", name.as_str());
        }
    });

    // 5. Initial Render
    render_combat_view(&session);
    render_roster_status(&session);

    // 6. Main Simulation Loop
    println!("\n[Engine] Starting Turn-Based Simulation Loop (Mode: {})...",
        if interactive { "INTERACTIVE STEP" } else { "AUTOPLAY" }
    );

    let mut outcome = BattleOutcome::Ongoing;
    let mut stdin_reader = io::stdin().lock();

    while outcome == BattleOutcome::Ongoing && session.current_round <= session.max_rounds {
        if interactive {
            print!("\n[Round {}/{}] Press [Enter] to advance, [g] grid, [r] roster, [q] forfeit: ",
                session.current_round, session.max_rounds);
            let _ = io::stdout().flush();
            let mut input = String::new();
            if stdin_reader.read_line(&mut input).is_err() || input.trim() == "q" {
                println!("[Game] Forfeiting encounter.");
                break;
            }
            let trimmed = input.trim();
            if trimmed == "g" {
                render_combat_view(&session);
                continue;
            } else if trimmed == "r" {
                render_roster_status(&session);
                continue;
            }
        }

        println!("\n>>> Advancing Round {}/{}...", session.current_round, session.max_rounds);
        outcome = session.resolve_round_autoplay();
        render_roster_status(&session);
    }

    // 7. Verdict & Statistics
    println!("\n==================================================================");
    match outcome {
        BattleOutcome::Victory => {
            println!("=== VICTORY! The party cleared '{}'! ===", encounter_name);
        }
        BattleOutcome::Defeat => {
            println!("=== DEFEAT! All party members have fallen. ===");
        }
        BattleOutcome::Ongoing => {
            println!("=== DRAW / TIMEOUT: Encounter concluded after maximum rounds. ===");
        }
    }
    println!("==================================================================");

    // 8. Autosave Game State
    let save_data = serde_json::json!({
        "engine": "Project Aegis v0.1",
        "format": "aegis_save",
        "version": 1,
        "world": "ashlands",
        "encounter": encounter_id,
        "round": session.current_round,
        "outcome": format!("{:?}", outcome),
        "heroes": session.allies.iter().map(|a| serde_json::json!({
            "id": a.id,
            "name": a.name,
            "hp": a.current_hp,
            "max_hp": a.max_hp,
            "attack": a.attack,
            "armor": a.armor,
            "speed": a.speed,
            "alive": a.is_alive()
        })).collect::<Vec<_>>(),
        "enemies": session.enemies.iter().map(|e| serde_json::json!({
            "id": e.id,
            "name": e.name,
            "hp": e.current_hp,
            "max_hp": e.max_hp,
            "alive": e.is_alive()
        })).collect::<Vec<_>>()
    });

    let save_path = "user://saves/slot_0.json";
    vfs.write_json(save_path, &save_data)?;
    println!("[Engine] Autosaved game state to '{}'", save_path);
    println!("[Engine] Total game instance runtime: {:.2} ms", start_time.elapsed().as_secs_f64() * 1000.0);

    Ok(())
}

fn print_help() {
    println!("Project Aegis - Game Instance Runner");
    println!("Usage: cargo run -p aegis_game -- [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --encounter <ID>   Select encounter (e.g. 'goblin_skirmish', 'veterans_drill')");
    println!("  --interactive, -i  Step through rounds interactively via keyboard");
    println!("  --seed <U64>       Specify combat RNG seed (default: 1337)");
    println!("  --load <PATH>      Inspect an existing game save file");
    println!("  --help, -h         Display this help message");
}

fn get_arg_value(args: &[String], flag: &str) -> Option<String> {
    if let Some(pos) = args.iter().position(|a| a == flag) {
        args.get(pos + 1).cloned()
    } else {
        None
    }
}

fn load_encounter_meta(catalog: &CatalogStore, id: &str) -> (String, Vec<String>, Vec<String>, i32) {
    if let Some(entry) = catalog.get_entry("encounters", id) {
        let name = entry.get_key("name").as_str().to_string();
        let max_rounds = entry.get_key("max_rounds").to_int();
        let party_ids = match entry.get_key("party") {
            Variant::Array(arr) => arr.iter().map(|v| v.as_str().to_string()).collect(),
            _ => vec!["warrior".to_string(), "ranger".to_string()],
        };
        let enemy_ids = match entry.get_key("enemies") {
            Variant::Array(arr) => arr.iter().map(|v| v.as_str().to_string()).collect(),
            _ => vec!["goblin_brawler".to_string(), "goblin_skirmisher".to_string()],
        };
        let mr = if max_rounds > 0 { max_rounds as i32 } else { 6 };
        (name, party_ids, enemy_ids, mr)
    } else {
        // Fallback default encounter
        (
            "Default Skirmish".to_string(),
            vec!["warrior".to_string(), "ranger".to_string(), "priest".to_string()],
            vec![
                "goblin_brawler".to_string(),
                "goblin_skirmisher".to_string(),
                "goblin_archer".to_string(),
                "goblin_shaman".to_string(),
                "goblin_berserker".to_string(),
            ],
            10,
        )
    }
}

fn resolve_unit(id: &str, team: Team, catalog: &CatalogStore) -> Combatant {
    let entry = catalog.get_entry("units", id);
    let (name, class_id, level) = if let Some(e) = entry {
        (
            e.get_key("name").as_str().to_string(),
            e.get_key("class_id").as_str().to_string(),
            e.get_key("level").to_int() as i32,
        )
    } else {
        (id.to_string(), "warrior".to_string(), 3)
    };

    // Calculate attributes calibrated to the reference balance dataset
    let (hp, atk, arm, spd) = match id {
        "warrior" | "dojo_warrior" => (242, 41, 18, 10),
        "ranger" | "dojo_ranger" => (158, 35, 6, 12),
        "priest" => (176, 34, 5, 9),
        "goblin_brawler" | "enemy_brawler" => (108, 31, 9, 8),
        "goblin_skirmisher" | "fleeing_scout" => (65, 32, 7, 14),
        "goblin_archer" => (71, 35, 7, 13),
        "goblin_shaman" => (77, 25, 6, 7),
        "goblin_berserker" | "ambush_berserker" => (108, 31, 5, 11),
        _ => match class_id.as_str() {
            "warrior" | "brawler" => (120 + level * 20, 25 + level * 3, 10, 8),
            "ranger" | "rogue" => (90 + level * 14, 22 + level * 3, 5, 12),
            "priest" | "mage" => (80 + level * 15, 20 + level * 2, 4, 8),
            _ => (100 + level * 15, 25 + level * 2, 6, 10),
        },
    };

    let display_name = if name.is_empty() { id.to_string() } else { name };
    Combatant::new(id, &display_name, team, hp, atk, arm, spd)
}

fn load_and_inspect_save(vfs: &Vfs, path: &str) -> anyhow::Result<()> {
    println!("[Engine] Loading game save from '{}'...", path);
    let val = vfs.read_json(path)?;
    println!("--------------------------------------------------");
    println!("Engine:    {}", val.get("engine").and_then(|v| v.as_str()).unwrap_or("Unknown"));
    println!("World:     {}", val.get("world").and_then(|v| v.as_str()).unwrap_or("Unknown"));
    println!("Encounter: {}", val.get("encounter").and_then(|v| v.as_str()).unwrap_or("Unknown"));
    println!("Round:     {}", val.get("round").and_then(|v| v.as_i64()).unwrap_or(0));
    println!("Outcome:   {}", val.get("outcome").and_then(|v| v.as_str()).unwrap_or("Unknown"));
    println!("--------------------------------------------------");
    if let Some(heroes) = val.get("heroes").and_then(|v| v.as_array()) {
        println!("Heroes in Party:");
        for h in heroes {
            let name = h.get("name").and_then(|v| v.as_str()).unwrap_or("Hero");
            let hp = h.get("hp").and_then(|v| v.as_i64()).unwrap_or(0);
            let max_hp = h.get("max_hp").and_then(|v| v.as_i64()).unwrap_or(0);
            let alive = h.get("alive").and_then(|v| v.as_bool()).unwrap_or(false);
            let status = if alive { format!("{}/{} HP", hp, max_hp) } else { "FALLEN".to_string() };
            println!("  * {:<16} : {}", name, status);
        }
    }
    println!("--------------------------------------------------");
    Ok(())
}

fn render_combat_view(session: &CombatSession) {
    println!("\n+-------------------------------------------------------+");
    println!("|              TACTICAL COMBAT BATTLEFIELD (12x9)       |");
    println!("+-------------------------------------------------------+");
    println!("   0  1  2  3  4  5  6  7  8  9 10 11  (Columns)");

    for y in 0..CombatGrid::GRID_H {
        print!("{:2} ", y);
        for x in 0..CombatGrid::GRID_W {
            let coord = IVec2::new(x, y);

            if let Some(ally) = session.allies.iter().find(|a| a.position == coord && a.is_alive()) {
                let badge = match ally.id.as_str() {
                    "warrior" | "dojo_warrior" => "[W]",
                    "ranger" | "dojo_ranger" => "[R]",
                    "priest" => "[P]",
                    _ => "[A]",
                };
                print!("{}", badge);
            } else if let Some(enemy) = session.enemies.iter().find(|e| e.position == coord && e.is_alive()) {
                let badge = match enemy.id.as_str() {
                    "goblin_brawler" | "enemy_brawler" => "<B>",
                    "goblin_skirmisher" | "fleeing_scout" => "<S>",
                    "goblin_archer" => "<A>",
                    "goblin_shaman" => "<M>",
                    "goblin_berserker" | "ambush_berserker" => "<Z>",
                    _ => "<E>",
                };
                print!("{}", badge);
            } else if x == 5 && y == 4 {
                print!(" | ");
            } else if x == 7 && y == 4 {
                print!(" | ");
            } else {
                print!(" . ");
            }
        }
        println!();
    }
    println!("+-------------------------------------------------------+");
    println!("Legend: [W]=Warrior, [R]=Ranger, [P]=Priest | <B/S/A/M/Z>=Goblins");
}

fn render_roster_status(session: &CombatSession) {
    println!("\n  --- Party Status ---");
    for ally in &session.allies {
        let status = if ally.is_alive() {
            format!("{}/{} HP", ally.current_hp, ally.max_hp)
        } else {
            "DOWN".to_string()
        };
        println!("    {:<16} : {:<10} @ pos {}", ally.name, status, ally.position);
    }
    println!("  --- Enemy Status ---");
    for enemy in &session.enemies {
        let status = if enemy.is_alive() {
            format!("{}/{} HP", enemy.current_hp, enemy.max_hp)
        } else {
            "DOWN".to_string()
        };
        println!("    {:<18} : {:<10} @ pos {}", enemy.name, status, enemy.position);
    }
}
