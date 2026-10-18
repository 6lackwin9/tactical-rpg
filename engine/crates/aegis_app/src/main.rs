use std::time::{Duration, Instant};
use egui::{Color32, FontId, Pos2, Rect, Rounding, Stroke, Vec2};
use aegis_core::IVec2;
use aegis_vm::Team;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    Offensive,
    Tactical,
    Defensive,
}

impl Stance {
    pub fn name(&self) -> &'static str {
        match self {
            Stance::Offensive => "Offensive",
            Stance::Tactical => "Tactical",
            Stance::Defensive => "Defensive",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Stance::Offensive => "⚔",
            Stance::Tactical => "⚡",
            Stance::Defensive => "🛡",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            Stance::Offensive => Color32::from_rgb(239, 68, 68),
            Stance::Tactical => Color32::from_rgb(56, 189, 248),
            Stance::Defensive => Color32::from_rgb(34, 197, 94),
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            Stance::Offensive => Stance::Tactical,
            Stance::Tactical => Stance::Defensive,
            Stance::Defensive => Stance::Offensive,
        }
    }

    pub fn multiplier_vs(&self, other: Stance) -> f32 {
        // Godot combat rule: Offensive > Tactical > Defensive > Offensive (1.25 / 0.75 / 1.0)
        match (self, other) {
            (Stance::Offensive, Stance::Tactical) => 1.25,
            (Stance::Tactical, Stance::Defensive) => 1.25,
            (Stance::Defensive, Stance::Offensive) => 1.25,
            (Stance::Offensive, Stance::Defensive) => 0.75,
            (Stance::Tactical, Stance::Offensive) => 0.75,
            (Stance::Defensive, Stance::Tactical) => 0.75,
            _ => 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: String,
    pub name: String,
    pub role: String,
    pub team: Team,
    pub max_hp: i32,
    pub current_hp: i32,
    pub attack: i32,
    pub armor: i32,
    pub speed: i32,
    pub range: i32,
    pub ap: i32,
    pub max_ap: i32,
    pub stance: Stance,
    pub pos: IVec2,
    pub acted: bool,
    pub guarding: bool,
    pub intent: String,
    pub intent_target_idx: Option<usize>,
}

impl Unit {
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn take_damage(&mut self, incoming: i32, attacker_stance: Stance) -> i32 {
        let eff_armor = if self.guarding { self.armor * 3 / 2 } else { self.armor };
        let mult = attacker_stance.multiplier_vs(self.stance);
        let base_power = (incoming as f32 * mult) as i32;
        let dmg = (base_power - eff_armor).max(1);
        self.current_hp = (self.current_hp - dmg).max(0);
        dmg
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let restored = (self.max_hp - self.current_hp).min(amount);
        self.current_hp += restored;
        restored
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMode {
    None,
    Move,
    Strike,
    Skill,
    Guard,
    Hold,
}

#[derive(Debug, Clone)]
pub struct FloatingText {
    pub text: String,
    pub color: Color32,
    pub grid_pos: IVec2,
    pub spawned: Instant,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct CombatLogEntry {
    pub round: i32,
    pub block: i32,
    pub text: String,
    pub color: Color32,
}

pub struct AegisApp {
    units: Vec<Unit>,
    round: i32,
    max_rounds: i32,
    current_block: i32,
    total_blocks: i32,
    selected_unit_idx: Option<usize>,
    command_mode: CommandMode,
    hovered_tile: Option<IVec2>,
    hovered_unit_idx: Option<usize>,
    floating_texts: Vec<FloatingText>,
    combat_log: Vec<CombatLogEntry>,
    autoplay: bool,
    last_autoplay_step: Instant,
    outcome: Option<String>,
    tactic_preset: String,
    obstacles: Vec<IVec2>,
    exit_tile: IVec2,
}

impl AegisApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            units: Vec::new(),
            round: 1,
            max_rounds: 10,
            current_block: 1,
            total_blocks: 6,
            selected_unit_idx: Some(0),
            command_mode: CommandMode::None,
            hovered_tile: None,
            hovered_unit_idx: None,
            floating_texts: Vec::new(),
            combat_log: Vec::new(),
            autoplay: false,
            last_autoplay_step: Instant::now(),
            outcome: None,
            tactic_preset: "Tactical".to_string(),
            obstacles: vec![
                IVec2::new(5, 1),
                IVec2::new(5, 7),
                IVec2::new(6, 1),
                IVec2::new(6, 7),
            ],
            exit_tile: IVec2::new(0, 4),
        };
        app.reset_battle();
        app
    }

    pub fn reset_battle(&mut self) {
        self.round = 1;
        self.current_block = 1;
        self.outcome = None;
        self.autoplay = false;
        self.command_mode = CommandMode::None;
        self.floating_texts.clear();
        self.combat_log.clear();

        self.units = vec![
            Unit {
                id: "warrior".to_string(),
                name: "Alda".to_string(),
                role: "Warrior (Tank)".to_string(),
                team: Team::Allies,
                max_hp: 242,
                current_hp: 242,
                attack: 41,
                armor: 18,
                speed: 10,
                range: 1,
                ap: 4,
                max_ap: 4,
                stance: Stance::Defensive,
                pos: IVec2::new(4, 4),
                acted: false,
                guarding: false,
                intent: String::new(),
                intent_target_idx: None,
            },
            Unit {
                id: "ranger".to_string(),
                name: "Bryn".to_string(),
                role: "Ranger (DPS)".to_string(),
                team: Team::Allies,
                max_hp: 158,
                current_hp: 158,
                attack: 35,
                armor: 6,
                speed: 12,
                range: 3,
                ap: 4,
                max_ap: 4,
                stance: Stance::Offensive,
                pos: IVec2::new(3, 2),
                acted: false,
                guarding: false,
                intent: String::new(),
                intent_target_idx: None,
            },
            Unit {
                id: "priest".to_string(),
                name: "Dagny".to_string(),
                role: "Priest (Healer)".to_string(),
                team: Team::Allies,
                max_hp: 176,
                current_hp: 176,
                attack: 34,
                armor: 5,
                speed: 9,
                range: 2,
                ap: 4,
                max_ap: 4,
                stance: Stance::Tactical,
                pos: IVec2::new(3, 6),
                acted: false,
                guarding: false,
                intent: String::new(),
                intent_target_idx: None,
            },
            // Hostiles
            Unit {
                id: "goblin_brawler".to_string(),
                name: "Goblin Brawler".to_string(),
                role: "Brute".to_string(),
                team: Team::Enemies,
                max_hp: 108,
                current_hp: 108,
                attack: 31,
                armor: 9,
                speed: 8,
                range: 1,
                ap: 3,
                max_ap: 3,
                stance: Stance::Offensive,
                pos: IVec2::new(7, 3),
                acted: false,
                guarding: false,
                intent: "Strikes Alda [B2]".to_string(),
                intent_target_idx: Some(0),
            },
            Unit {
                id: "goblin_skirmisher".to_string(),
                name: "Goblin Skirmisher".to_string(),
                role: "Scout".to_string(),
                team: Team::Enemies,
                max_hp: 65,
                current_hp: 65,
                attack: 32,
                armor: 7,
                speed: 14,
                range: 1,
                ap: 4,
                max_ap: 4,
                stance: Stance::Tactical,
                pos: IVec2::new(8, 2),
                acted: false,
                guarding: false,
                intent: "Flanks Bryn [B1]".to_string(),
                intent_target_idx: Some(1),
            },
            Unit {
                id: "goblin_archer".to_string(),
                name: "Goblin Archer".to_string(),
                role: "Ranged".to_string(),
                team: Team::Enemies,
                max_hp: 71,
                current_hp: 71,
                attack: 35,
                armor: 7,
                speed: 13,
                range: 3,
                ap: 3,
                max_ap: 3,
                stance: Stance::Tactical,
                pos: IVec2::new(9, 4),
                acted: false,
                guarding: false,
                intent: "Snipes Bryn [B2]".to_string(),
                intent_target_idx: Some(1),
            },
            Unit {
                id: "goblin_shaman".to_string(),
                name: "Goblin Shaman".to_string(),
                role: "Caster".to_string(),
                team: Team::Enemies,
                max_hp: 77,
                current_hp: 77,
                attack: 25,
                armor: 6,
                speed: 7,
                range: 2,
                ap: 4,
                max_ap: 4,
                stance: Stance::Tactical,
                pos: IVec2::new(9, 6),
                acted: false,
                guarding: false,
                intent: "Casts Poison on Alda [B3]".to_string(),
                intent_target_idx: Some(0),
            },
            Unit {
                id: "goblin_berserker".to_string(),
                name: "Goblin Berserker".to_string(),
                role: "Heavy".to_string(),
                team: Team::Enemies,
                max_hp: 108,
                current_hp: 108,
                attack: 31,
                armor: 5,
                speed: 11,
                range: 1,
                ap: 4,
                max_ap: 4,
                stance: Stance::Offensive,
                pos: IVec2::new(7, 5),
                acted: false,
                guarding: false,
                intent: "Cleaves Alda [B2]".to_string(),
                intent_target_idx: Some(0),
            },
        ];

        self.selected_unit_idx = Some(0);
        self.update_enemy_intents();
        self.add_log(format!("--- Encounter Initialized: Arena 12x9 (COMBAT-UI2-L1 Layout C) ---"), Color32::from_rgb(148, 163, 184));
        self.add_log(format!("Timeline: 6 Blocks per Round | Stance Matchups Active (Offensive > Tactical > Defensive)"), Color32::from_rgb(56, 189, 248));
    }

    fn add_log(&mut self, text: String, color: Color32) {
        self.combat_log.push(CombatLogEntry {
            round: self.round,
            block: self.current_block,
            text,
            color,
        });
        if self.combat_log.len() > 100 {
            self.combat_log.remove(0);
        }
    }

    fn add_float(&mut self, text: String, color: Color32, pos: IVec2) {
        self.floating_texts.push(FloatingText {
            text,
            color,
            grid_pos: pos,
            spawned: Instant::now(),
            duration: Duration::from_millis(1600),
        });
    }

    fn find_unit_at(&self, pos: IVec2) -> Option<usize> {
        self.units.iter().position(|u| u.is_alive() && u.pos == pos)
    }

    fn is_obstacle(&self, pos: IVec2) -> bool {
        self.obstacles.contains(&pos)
    }

    fn update_enemy_intents(&mut self) {
        let hero_indices: Vec<usize> = self.units.iter().enumerate()
            .filter(|(_, u)| u.team == Team::Allies && u.is_alive())
            .map(|(i, _)| i)
            .collect();

        if hero_indices.is_empty() {
            return;
        }

        let enemy_indices: Vec<usize> = self.units.iter().enumerate()
            .filter(|(_, u)| u.team == Team::Enemies && u.is_alive())
            .map(|(i, _)| i)
            .collect();

        for e_idx in enemy_indices {
            let e_pos = self.units[e_idx].pos;
            let target_h_idx = hero_indices.iter()
                .min_by_key(|&&h_idx| self.units[h_idx].pos.manhattan_distance(e_pos))
                .copied()
                .unwrap_or(hero_indices[0]);

            let target_name = self.units[target_h_idx].name.clone();
            let dist = self.units[target_h_idx].pos.manhattan_distance(e_pos);
            let rng = self.units[e_idx].range;

            self.units[e_idx].intent_target_idx = Some(target_h_idx);
            if dist <= rng {
                self.units[e_idx].intent = format!("Strikes {} [B2]", target_name);
            } else {
                self.units[e_idx].intent = format!("Advances on {} [B1]", target_name);
            }
        }
    }

    fn check_victory(&mut self) {
        let allies_alive = self.units.iter().any(|u| u.team == Team::Allies && u.is_alive());
        let enemies_alive = self.units.iter().any(|u| u.team == Team::Enemies && u.is_alive());

        if !allies_alive {
            self.outcome = Some("DEFEAT - Your party has been wiped out.".to_string());
            self.autoplay = false;
            self.add_log("=== BATTLE TERMINATED: DEFEAT ===".to_string(), Color32::from_rgb(239, 68, 68));
        } else if !enemies_alive {
            self.outcome = Some("VICTORY - All hostile forces eliminated!".to_string());
            self.autoplay = false;
            self.add_log("=== BATTLE WON: TACTICAL VICTORY! ===".to_string(), Color32::from_rgb(34, 197, 94));
        } else if self.round > self.max_rounds {
            self.outcome = Some("DRAW - Maximum combat rounds elapsed.".to_string());
            self.autoplay = false;
        }
    }

    pub fn cycle_hero_stance(&mut self, hero_idx: usize) {
        if let Some(hero) = self.units.get_mut(hero_idx) {
            if hero.ap >= 1 {
                hero.ap -= 1;
                let old_stance = hero.stance;
                hero.stance = hero.stance.cycle();
                let name = hero.name.clone();
                let new_stance = hero.stance;
                let pos = hero.pos;
                let cur_ap = hero.ap;
                let max_ap = hero.max_ap;
                self.add_log(format!("{} shifts stance: {} -> {} (-1 AP, remaining: {}/{})", name, old_stance.name(), new_stance.name(), cur_ap, max_ap), Color32::from_rgb(250, 204, 21));
                self.add_float(format!("Stance: {}", new_stance.name()), new_stance.color(), pos);
            }
        }
    }

    pub fn execute_move(&mut self, hero_idx: usize, target: IVec2) {
        if self.is_obstacle(target) {
            return;
        }
        if let Some(hero) = self.units.get_mut(hero_idx) {
            let old_pos = hero.pos;
            hero.pos = target;
            hero.acted = true;
            let name = hero.name.clone();
            self.add_log(format!("{} advances from ({},{}) to ({},{})", name, old_pos.x, old_pos.y, target.x, target.y), Color32::from_rgb(56, 189, 248));
            self.add_float("Moved".to_string(), Color32::from_rgb(56, 189, 248), target);
        }
        self.command_mode = CommandMode::None;
        self.check_end_of_player_turn();
    }

    pub fn execute_attack(&mut self, hero_idx: usize, target_idx: usize) {
        let (hero_name, hero_atk, hero_stance, hero_pos) = {
            let h = &self.units[hero_idx];
            (h.name.clone(), h.attack, h.stance, h.pos)
        };
        let (enemy_name, enemy_pos, enemy_stance, dmg, remaining_hp, enemy_alive, counter_dmg) = {
            let enemy = &mut self.units[target_idx];
            let dmg = enemy.take_damage(hero_atk, hero_stance);
            let alive = enemy.is_alive();
            let remaining = enemy.current_hp;
            let counter = if alive && (enemy.pos.manhattan_distance(hero_pos) <= enemy.range) {
                Some(enemy.attack)
            } else {
                None
            };
            (enemy.name.clone(), enemy.pos, enemy.stance, dmg, remaining, alive, counter)
        };

        self.units[hero_idx].acted = true;
        let mult = hero_stance.multiplier_vs(enemy_stance);
        let stance_note = if mult > 1.0 { " (Stance Advantage +25%!)" } else if mult < 1.0 { " (Stance Disadvantage -25%)" } else { "" };
        self.add_log(format!("⚔ {} [{}] strikes {} [{}] for {} DMG!{}{}", hero_name, hero_stance.name(), enemy_name, enemy_stance.name(), dmg, stance_note, format!(" (Target HP: {})", remaining_hp)), Color32::from_rgb(250, 204, 21));
        self.add_float(format!("-{}", dmg), Color32::from_rgb(239, 68, 68), enemy_pos);

        if !enemy_alive {
            self.add_log(format!("💀 {} has been slain!", enemy_name), Color32::from_rgb(239, 68, 68));
            self.add_float("DEFEATED".to_string(), Color32::from_rgb(244, 63, 94), enemy_pos);
        } else if let Some(catk) = counter_dmg {
            let (actual_cdmg, cur_hero_hp) = {
                let hero = &mut self.units[hero_idx];
                let actual = hero.take_damage(catk, enemy_stance);
                (actual, hero.current_hp)
            };
            self.add_log(format!("↩ {} retaliates against {} for {} DMG! (Hero HP: {})", enemy_name, hero_name, actual_cdmg, cur_hero_hp), Color32::from_rgb(248, 113, 113));
            self.add_float(format!("-{}", actual_cdmg), Color32::from_rgb(248, 113, 113), hero_pos);
        }

        self.command_mode = CommandMode::None;
        self.check_victory();
        self.check_end_of_player_turn();
    }

    pub fn execute_skill(&mut self, hero_idx: usize, target_idx: Option<usize>) {
        let (hero_id, hero_name, hero_atk, hero_stance) = {
            let h = &self.units[hero_idx];
            (h.id.clone(), h.name.clone(), h.attack, h.stance)
        };

        match hero_id.as_str() {
            "warrior" => {
                // Shield Bash: Heavy blow with +15 power
                if let Some(t_idx) = target_idx {
                    let (dmg, e_name, e_pos, alive) = {
                        let enemy = &mut self.units[t_idx];
                        let dmg = enemy.take_damage(hero_atk + 15, hero_stance);
                        (dmg, enemy.name.clone(), enemy.pos, enemy.is_alive())
                    };
                    self.units[hero_idx].acted = true;
                    self.add_log(format!("💥 {} executes [SHIELD BASH] on {} for {} DMG!", hero_name, e_name, dmg), Color32::from_rgb(251, 146, 60));
                    self.add_float(format!("BASH! -{}", dmg), Color32::from_rgb(251, 146, 60), e_pos);
                    if !alive {
                        self.add_log(format!("💀 {} has been slain!", e_name), Color32::from_rgb(239, 68, 68));
                    }
                }
            }
            "ranger" => {
                // Snipe: High damage at long range
                if let Some(t_idx) = target_idx {
                    let (dmg, e_name, e_pos, alive) = {
                        let enemy = &mut self.units[t_idx];
                        let dmg = enemy.take_damage(hero_atk + 12, hero_stance);
                        (dmg, enemy.name.clone(), enemy.pos, enemy.is_alive())
                    };
                    self.units[hero_idx].acted = true;
                    self.add_log(format!("🎯 {} fires [SNIPE] piercing {} for {} DMG!", hero_name, e_name, dmg), Color32::from_rgb(168, 85, 247));
                    self.add_float(format!("SNIPE! -{}", dmg), Color32::from_rgb(168, 85, 247), e_pos);
                    if !alive {
                        self.add_log(format!("💀 {} has been slain!", e_name), Color32::from_rgb(239, 68, 68));
                    }
                }
            }
            "priest" => {
                // Divine Mend: Heals lowest ally 45 HP
                let lowest_ally_idx = self.units.iter().enumerate()
                    .filter(|(_, u)| u.team == Team::Allies && u.is_alive())
                    .min_by_key(|(_, u)| u.current_hp)
                    .map(|(i, _)| i);

                if let Some(a_idx) = lowest_ally_idx {
                    let ally = &mut self.units[a_idx];
                    let healed = ally.heal(45);
                    let a_name = ally.name.clone();
                    let a_pos = ally.pos;
                    self.units[hero_idx].acted = true;
                    self.add_log(format!("✨ {} casts [DIVINE MEND] on {} restoring {} HP!", hero_name, a_name, healed), Color32::from_rgb(34, 197, 94));
                    self.add_float(format!("+{} HP", healed), Color32::from_rgb(34, 197, 94), a_pos);
                }
            }
            _ => {}
        }

        self.command_mode = CommandMode::None;
        self.check_victory();
        self.check_end_of_player_turn();
    }

    pub fn execute_guard(&mut self, hero_idx: usize) {
        if let Some(hero) = self.units.get_mut(hero_idx) {
            hero.guarding = true;
            hero.acted = true;
            let name = hero.name.clone();
            let pos = hero.pos;
            self.add_log(format!("🛡 {} assumes [GUARD STANCE] (+50% Armor rating)!", name), Color32::from_rgb(148, 163, 184));
            self.add_float("GUARD".to_string(), Color32::from_rgb(148, 163, 184), pos);
        }
        self.command_mode = CommandMode::None;
        self.check_end_of_player_turn();
    }

    pub fn execute_hold(&mut self, hero_idx: usize) {
        if let Some(hero) = self.units.get_mut(hero_idx) {
            hero.acted = true;
            let name = hero.name.clone();
            self.add_log(format!("⏳ {} holds current position.", name), Color32::from_rgb(100, 116, 139));
        }
        self.command_mode = CommandMode::None;
        self.check_end_of_player_turn();
    }

    fn check_end_of_player_turn(&mut self) {
        let all_allies_acted = self.units.iter()
            .filter(|u| u.team == Team::Allies && u.is_alive())
            .all(|u| u.acted);

        if all_allies_acted {
            self.resolve_enemy_turn();
        }
    }

    fn resolve_enemy_turn(&mut self) {
        self.add_log(format!("--- Hostile Phase Commences ---"), Color32::from_rgb(239, 68, 68));

        let enemy_indices: Vec<usize> = self.units.iter().enumerate()
            .filter(|(_, u)| u.team == Team::Enemies && u.is_alive())
            .map(|(i, _)| i)
            .collect();

        for e_idx in enemy_indices {
            if !self.units[e_idx].is_alive() {
                continue;
            }

            // Find closest live ally
            let e_pos = self.units[e_idx].pos;
            let target_ally_idx = self.units.iter().enumerate()
                .filter(|(_, u)| u.team == Team::Allies && u.is_alive())
                .min_by_key(|(_, u)| u.pos.manhattan_distance(e_pos))
                .map(|(i, _)| i);

            if let Some(a_idx) = target_ally_idx {
                let a_pos = self.units[a_idx].pos;
                let dist = e_pos.manhattan_distance(a_pos);
                let e_range = self.units[e_idx].range;

                if dist > e_range {
                    // Step closer to target ally
                    let mut step = e_pos;
                    if e_pos.x > a_pos.x { step.x -= 1; }
                    else if e_pos.x < a_pos.x { step.x += 1; }
                    else if e_pos.y > a_pos.y { step.y -= 1; }
                    else if e_pos.y < a_pos.y { step.y += 1; }

                    if !self.is_obstacle(step) && self.find_unit_at(step).is_none() && step.x >= 0 && step.x < 12 && step.y >= 0 && step.y < 9 {
                        self.units[e_idx].pos = step;
                    }
                }

                // If now in attack range, strike!
                let cur_dist = self.units[e_idx].pos.manhattan_distance(self.units[a_idx].pos);
                if cur_dist <= self.units[e_idx].range {
                    let e_atk = self.units[e_idx].attack;
                    let e_name = self.units[e_idx].name.clone();
                    let e_stance = self.units[e_idx].stance;
                    let (dmg, a_name, a_pos, cur_a_hp, alive) = {
                        let ally = &mut self.units[a_idx];
                        let dmg = ally.take_damage(e_atk, e_stance);
                        (dmg, ally.name.clone(), ally.pos, ally.current_hp, ally.is_alive())
                    };
                    self.add_log(format!("⚔ {} [{}] attacks {} for {} DMG! (Hero HP: {})", e_name, e_stance.name(), a_name, dmg, cur_a_hp), Color32::from_rgb(248, 113, 113));
                    self.add_float(format!("-{}", dmg), Color32::from_rgb(239, 68, 68), a_pos);
                    if !alive {
                        self.add_log(format!("💀 Hero {} has fallen!", a_name), Color32::from_rgb(239, 68, 68));
                    }
                }
            }
        }

        // Reset turn flags for new round
        self.round += 1;
        self.current_block = 1;
        for u in self.units.iter_mut() {
            u.acted = false;
            u.guarding = false;
            u.ap = u.max_ap;
        }

        self.update_enemy_intents();
        self.add_log(format!("=== Round {} Begins: Player Phase (6 Blocks) ===", self.round), Color32::from_rgb(56, 189, 248));
        self.check_victory();
    }

    pub fn step_autoplay_round(&mut self) {
        if self.outcome.is_some() {
            return;
        }

        // Auto ally actions
        let ally_indices: Vec<usize> = self.units.iter().enumerate()
            .filter(|(_, u)| u.team == Team::Allies && u.is_alive() && !u.acted)
            .map(|(i, _)| i)
            .collect();

        for a_idx in ally_indices {
            let a_pos = self.units[a_idx].pos;
            let target_enemy_idx = self.units.iter().enumerate()
                .filter(|(_, u)| u.team == Team::Enemies && u.is_alive())
                .min_by_key(|(_, u)| u.pos.manhattan_distance(a_pos))
                .map(|(i, _)| i);

            if let Some(e_idx) = target_enemy_idx {
                let e_pos = self.units[e_idx].pos;
                let dist = a_pos.manhattan_distance(e_pos);
                let rng = self.units[a_idx].range;

                if dist <= rng {
                    self.execute_attack(a_idx, e_idx);
                } else {
                    // Move 1 step towards enemy
                    let mut step = a_pos;
                    if a_pos.x < e_pos.x { step.x += 1; }
                    else if a_pos.x > e_pos.x { step.x -= 1; }
                    else if a_pos.y < e_pos.y { step.y += 1; }
                    else if a_pos.y > e_pos.y { step.y += 1; }

                    if !self.is_obstacle(step) && self.find_unit_at(step).is_none() && step.x >= 0 && step.x < 12 && step.y >= 0 && step.y < 9 {
                        self.execute_move(a_idx, step);
                    } else {
                        self.execute_hold(a_idx);
                    }
                }
            } else {
                self.execute_hold(a_idx);
            }
        }
    }
}

impl eframe::App for AegisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.step_autoplay_round();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::A)) {
            self.autoplay = !self.autoplay;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.reset_battle();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::M)) {
            self.command_mode = CommandMode::Move;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
            self.command_mode = CommandMode::Strike;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
            self.command_mode = CommandMode::Skill;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
            self.command_mode = CommandMode::Guard;
        }

        // Autoplay timer step
        if self.autoplay && self.outcome.is_none() {
            if self.last_autoplay_step.elapsed() >= Duration::from_millis(900) {
                self.step_autoplay_round();
                self.last_autoplay_step = Instant::now();
            }
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        // Clean up expired floating texts
        let now = Instant::now();
        self.floating_texts.retain(|f| now.duration_since(f.spawned) < f.duration);
        if !self.floating_texts.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }

        // Top Header Panel: Timeline Ruler & Action Controls (COMBAT-UI2-L1)
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(Color32::from_rgb(15, 23, 42)).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("⚔ PROJECT AEGIS").strong().color(Color32::from_rgb(241, 245, 249)));
                    ui.label(egui::RichText::new("[NATIVE 2D TACTICAL ENGINE - GODOT PARITY]").size(10.0).color(Color32::from_rgb(56, 189, 248)));

                    ui.separator();

                    // Round & Blocks Timeline Ruler (B1 to B6)
                    ui.label(egui::RichText::new(format!("Round {}/{}", self.round, self.max_rounds)).strong().color(Color32::from_rgb(250, 204, 21)));

                    ui.horizontal(|ui| {
                        for b in 1..=self.total_blocks {
                            let is_current = b == self.current_block;
                            let block_color = if is_current {
                                Color32::from_rgb(56, 189, 248)
                            } else {
                                Color32::from_rgb(71, 85, 105)
                            };
                            let block_bg = if is_current {
                                Color32::from_rgb(14, 116, 144)
                            } else {
                                Color32::from_rgb(30, 41, 59)
                            };

                            egui::Frame::none()
                                .fill(block_bg)
                                .stroke(Stroke::new(1.0_f32, block_color))
                                .rounding(Rounding::same(3.0))
                                .inner_margin(Vec2::new(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(format!("B{}", b)).size(10.0).color(if is_current { Color32::WHITE } else { Color32::LIGHT_GRAY }));
                                });
                        }
                    });

                    ui.separator();

                    if let Some(ref outcome) = self.outcome {
                        ui.label(egui::RichText::new(outcome).strong().color(Color32::from_rgb(244, 63, 94)));
                    } else {
                        ui.label(egui::RichText::new("● PLAYER PHASE").strong().color(Color32::from_rgb(34, 197, 94)));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("↺ Reset [R]").color(Color32::from_rgb(248, 113, 113))).clicked() {
                            self.reset_battle();
                        }

                        let auto_label = if self.autoplay { "⏸ Pause Auto [A]" } else { "⚡ Autoplay [A]" };
                        if ui.button(egui::RichText::new(auto_label).color(Color32::from_rgb(250, 204, 21))).clicked() {
                            self.autoplay = !self.autoplay;
                        }

                        if ui.button(egui::RichText::new("▶ Step Round [Space]").strong().color(Color32::from_rgb(56, 189, 248))).clicked() {
                            self.step_autoplay_round();
                        }

                        // Tactic Preset Dropdown / Button
                        if ui.button(egui::RichText::new(format!("Tactic: {}", self.tactic_preset)).size(11.0).color(Color32::from_rgb(203, 213, 225))).clicked() {
                            self.tactic_preset = match self.tactic_preset.as_str() {
                                "Tactical" => "Aggressive".to_string(),
                                "Aggressive" => "Cautious".to_string(),
                                _ => "Tactical".to_string(),
                            };
                        }
                    });
                });
            });

        // Left Panel: Party Roster & Command Dock
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .default_width(310.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(15, 23, 42)).inner_margin(12.0))
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("PARTY ROSTER (ALLIES)").size(14.0).color(Color32::from_rgb(56, 189, 248)));
                ui.add_space(4.0);

                let ally_indices: Vec<usize> = self.units.iter().enumerate()
                    .filter(|(_, u)| u.team == Team::Allies)
                    .map(|(i, _)| i)
                    .collect();

                let mut new_selected_unit_idx = None;
                let mut cycle_stance_idx = None;

                for idx in ally_indices {
                    let (u_name, u_role, u_id, u_hp, u_max_hp, u_pos, u_atk, u_def, u_rng, u_acted, u_stance, u_ap, u_max_ap, is_alive) = {
                        let u = &self.units[idx];
                        (u.name.clone(), u.role.clone(), u.id.clone(), u.current_hp, u.max_hp, u.pos, u.attack, u.armor, u.range, u.acted, u.stance, u.ap, u.max_ap, u.is_alive())
                    };
                    let is_selected = self.selected_unit_idx == Some(idx);

                    let bg_color = if !is_alive {
                        Color32::from_rgb(30, 41, 59)
                    } else if is_selected {
                        Color32::from_rgb(30, 58, 138)
                    } else {
                        Color32::from_rgb(23, 37, 84)
                    };

                    egui::Frame::none()
                        .fill(bg_color)
                        .stroke(Stroke::new(if is_selected { 2.0_f32 } else { 1.0_f32 }, if is_selected { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(51, 65, 85) }))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let badge_color = match u_id.as_str() {
                                    "warrior" => Color32::from_rgb(59, 130, 246),
                                    "ranger" => Color32::from_rgb(16, 185, 129),
                                    _ => Color32::from_rgb(234, 179, 8),
                                };
                                ui.label(egui::RichText::new(&u_name).strong().color(Color32::WHITE));
                                ui.label(egui::RichText::new(&u_role).size(10.0).color(badge_color));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button(if is_selected { "Selected" } else { "Select" }).clicked() {
                                        new_selected_unit_idx = Some(idx);
                                    }
                                });
                            });

                            let hp_pct = u_hp as f32 / u_max_hp as f32;
                            let hp_color = if hp_pct > 0.5 {
                                Color32::from_rgb(34, 197, 94)
                            } else if hp_pct > 0.25 {
                                Color32::from_rgb(234, 179, 8)
                            } else {
                                Color32::from_rgb(239, 68, 68)
                            };

                            ui.add(egui::ProgressBar::new(hp_pct).text(format!("{}/{} HP", u_hp, u_max_hp)).fill(hp_color));

                            ui.horizontal(|ui| {
                                // Stance Badge & Button
                                if ui.small_button(egui::RichText::new(format!("{} {}", u_stance.icon(), u_stance.name())).size(10.0).color(u_stance.color())).clicked() {
                                    cycle_stance_idx = Some(idx);
                                }

                                // AP pips
                                let mut pips = String::new();
                                for i in 0..u_max_ap {
                                    if i < u_ap { pips.push('●'); } else { pips.push('○'); }
                                }
                                ui.label(egui::RichText::new(format!("AP: {} ({}/{})", pips, u_ap, u_max_ap)).size(10.0).color(Color32::from_rgb(250, 204, 21)));
                            });

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("Pos: ({},{}) | ATK: {} | DEF: {} | RNG: {}", u_pos.x, u_pos.y, u_atk, u_def, u_rng)).size(10.0).color(Color32::from_rgb(148, 163, 184)));
                            });
                            if u_acted {
                                ui.label(egui::RichText::new("✓ Action Resolved").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                            } else if is_alive {
                                ui.label(egui::RichText::new("● Ready to Act").size(10.0).color(Color32::from_rgb(34, 197, 94)));
                            }
                        });

                    ui.add_space(4.0);
                }

                if let Some(n_idx) = new_selected_unit_idx {
                    self.selected_unit_idx = Some(n_idx);
                    self.command_mode = CommandMode::None;
                }
                if let Some(s_idx) = cycle_stance_idx {
                    self.cycle_hero_stance(s_idx);
                }

                ui.separator();
                ui.heading(egui::RichText::new("COMMAND DOCK").size(14.0).color(Color32::from_rgb(250, 204, 21)));

                if let Some(sel_idx) = self.selected_unit_idx {
                    let (sel_name, sel_role, sel_id, sel_stance) = {
                        let sel = &self.units[sel_idx];
                        (sel.name.clone(), sel.role.clone(), sel.id.clone(), sel.stance)
                    };
                    ui.label(egui::RichText::new(format!("Active: {} ({}) [{}]", sel_name, sel_role, sel_stance.name())).strong().color(Color32::WHITE));

                    ui.add_space(4.0);
                    let mut chosen_command = None;
                    let mut trigger_skill = false;
                    let mut trigger_guard = false;
                    let mut trigger_hold = false;

                    ui.horizontal_wrapped(|ui| {
                        let move_btn = ui.button(egui::RichText::new("🚶 Move [M]").color(if self.command_mode == CommandMode::Move { Color32::from_rgb(56, 189, 248) } else { Color32::WHITE }));
                        if move_btn.clicked() {
                            chosen_command = Some(CommandMode::Move);
                        }

                        let strike_btn = ui.button(egui::RichText::new("⚔ Strike [1]").color(if self.command_mode == CommandMode::Strike { Color32::from_rgb(239, 68, 68) } else { Color32::WHITE }));
                        if strike_btn.clicked() {
                            chosen_command = Some(CommandMode::Strike);
                        }

                        let skill_label = match sel_id.as_str() {
                            "warrior" => "💥 Shield Bash [2]",
                            "ranger" => "🎯 Snipe [2]",
                            _ => "✨ Divine Mend [2]",
                        };
                        let skill_btn = ui.button(egui::RichText::new(skill_label).color(if self.command_mode == CommandMode::Skill { Color32::from_rgb(168, 85, 247) } else { Color32::WHITE }));
                        if skill_btn.clicked() {
                            chosen_command = Some(CommandMode::Skill);
                            if sel_id == "priest" {
                                trigger_skill = true;
                            }
                        }

                        let guard_btn = ui.button(egui::RichText::new("🛡 Guard [3]").color(Color32::from_rgb(148, 163, 184)));
                        if guard_btn.clicked() {
                            trigger_guard = true;
                        }

                        let hold_btn = ui.button(egui::RichText::new("⏳ Hold").color(Color32::from_rgb(100, 116, 139)));
                        if hold_btn.clicked() {
                            trigger_hold = true;
                        }
                    });

                    if let Some(cmd) = chosen_command {
                        self.command_mode = cmd;
                    }
                    if trigger_skill {
                        self.execute_skill(sel_idx, None);
                    }
                    if trigger_guard {
                        self.execute_guard(sel_idx);
                    }
                    if trigger_hold {
                        self.execute_hold(sel_idx);
                    }

                    ui.add_space(6.0);
                    match self.command_mode {
                        CommandMode::Move => {
                            ui.label(egui::RichText::new("👉 Click on any highlighted cyan tile to MOVE.").size(11.0).color(Color32::from_rgb(56, 189, 248)));
                        }
                        CommandMode::Strike => {
                            ui.label(egui::RichText::new("👉 Click on any red hostile unit to ATTACK.").size(11.0).color(Color32::from_rgb(239, 68, 68)));
                        }
                        CommandMode::Skill => {
                            ui.label(egui::RichText::new("👉 Click target hostile unit to unleash SKILL.").size(11.0).color(Color32::from_rgb(168, 85, 247)));
                        }
                        _ => {}
                    }
                } else {
                    ui.label(egui::RichText::new("Select a hero from the roster or arena grid.").italics().color(Color32::from_rgb(100, 116, 139)));
                }
            });

        // Right Panel: Hostiles Roster & Live Combat Log
        egui::SidePanel::right("right_panel")
            .resizable(false)
            .default_width(320.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(15, 23, 42)).inner_margin(12.0))
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("HOSTILE FORCES (ENEMIES)").size(14.0).color(Color32::from_rgb(239, 68, 68)));
                ui.add_space(4.0);

                let enemy_indices: Vec<usize> = self.units.iter().enumerate()
                    .filter(|(_, u)| u.team == Team::Enemies)
                    .map(|(i, _)| i)
                    .collect();

                let mut target_enemy_idx: Option<usize> = None;

                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    for idx in enemy_indices {
                        let (u_name, u_role, u_hp, u_max_hp, u_pos, u_atk, u_def, u_stance, u_intent, is_alive) = {
                            let u = &self.units[idx];
                            (u.name.clone(), u.role.clone(), u.current_hp, u.max_hp, u.pos, u.attack, u.armor, u.stance, u.intent.clone(), u.is_alive())
                        };
                        let bg_color = if !is_alive {
                            Color32::from_rgb(30, 41, 59)
                        } else {
                            Color32::from_rgb(69, 10, 10)
                        };

                        egui::Frame::none()
                            .fill(bg_color)
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(127, 29, 29)))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(6.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&u_name).strong().color(if is_alive { Color32::WHITE } else { Color32::GRAY }));
                                    ui.label(egui::RichText::new(&u_role).size(10.0).color(Color32::from_rgb(248, 113, 113)));
                                    ui.label(egui::RichText::new(format!("[{}]", u_stance.name())).size(9.0).color(u_stance.color()));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if is_alive && ui.small_button("Target").clicked() {
                                            target_enemy_idx = Some(idx);
                                        }
                                    });
                                });

                                let hp_pct = u_hp as f32 / u_max_hp as f32;
                                ui.add(egui::ProgressBar::new(hp_pct).text(format!("{}/{} HP", u_hp, u_max_hp)).fill(Color32::from_rgb(239, 68, 68)));

                                if is_alive && !u_intent.is_empty() {
                                    ui.label(egui::RichText::new(format!("👁 Intent: {}", u_intent)).size(10.0).color(Color32::from_rgb(250, 204, 21)));
                                }

                                ui.label(egui::RichText::new(format!("Pos: ({},{}) | ATK: {} | DEF: {}", u_pos.x, u_pos.y, u_atk, u_def)).size(10.0).color(Color32::from_rgb(148, 163, 184)));
                            });
                        ui.add_space(2.0);
                    }
                });

                if let (Some(sel_idx), Some(t_idx)) = (self.selected_unit_idx, target_enemy_idx) {
                    self.execute_attack(sel_idx, t_idx);
                }

                ui.separator();
                ui.heading(egui::RichText::new("TACTICAL COMBAT LOG").size(14.0).color(Color32::from_rgb(250, 204, 21)));

                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for entry in &self.combat_log {
                        ui.label(egui::RichText::new(format!("[R{}:B{}] {}", entry.round, entry.block, entry.text)).size(11.0).color(entry.color));
                    }
                });
            });

        // Center Panel: Interactive 2D Tactical Arena Grid
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(11, 15, 25)).inner_margin(8.0))
            .show(ctx, |ui| {
                let available_size = ui.available_size();
                let grid_w = 12;
                let grid_h = 9;

                let cell_size = (available_size.x / grid_w as f32).min(available_size.y / grid_h as f32);
                let actual_w = cell_size * grid_w as f32;
                let actual_h = cell_size * grid_h as f32;

                let (response, painter) = ui.allocate_painter(Vec2::new(actual_w, actual_h), egui::Sense::click_and_drag());
                let grid_origin = response.rect.min;

                // Track hover tile
                self.hovered_tile = None;
                self.hovered_unit_idx = None;

                if let Some(hover_pos) = response.hover_pos() {
                    let rel_x = hover_pos.x - grid_origin.x;
                    let rel_y = hover_pos.y - grid_origin.y;
                    let tx = (rel_x / cell_size).floor() as i32;
                    let ty = (rel_y / cell_size).floor() as i32;

                    if tx >= 0 && tx < grid_w && ty >= 0 && ty < grid_h {
                        let tile = IVec2::new(tx, ty);
                        self.hovered_tile = Some(tile);
                        self.hovered_unit_idx = self.find_unit_at(tile);
                    }
                }

                // Selected hero stats for range overlays
                let (sel_pos, sel_range, sel_can_act) = if let Some(s_idx) = self.selected_unit_idx {
                    let u = &self.units[s_idx];
                    (Some(u.pos), u.range, !u.acted && u.is_alive())
                } else {
                    (None, 1, false)
                };

                // Draw Grid Tiles
                for x in 0..grid_w {
                    for y in 0..grid_h {
                        let tile = IVec2::new(x, y);
                        let tile_min = Pos2::new(grid_origin.x + x as f32 * cell_size, grid_origin.y + y as f32 * cell_size);
                        let tile_max = Pos2::new(tile_min.x + cell_size - 1.0, tile_min.y + cell_size - 1.0);
                        let tile_rect = Rect::from_two_pos(tile_min, tile_max);

                        // Checkerboard base color & terrain
                        let is_even = (x + y) % 2 == 0;
                        let is_obs = self.is_obstacle(tile);
                        let is_exit = tile == self.exit_tile;

                        let mut tile_color = if is_obs {
                            Color32::from_rgb(18, 18, 24)
                        } else if is_exit {
                            Color32::from_rgb(16, 48, 36)
                        } else if is_even {
                            Color32::from_rgb(26, 34, 51)
                        } else {
                            Color32::from_rgb(34, 43, 64)
                        };

                        let mut border_color = if is_obs {
                            Color32::from_rgb(40, 40, 50)
                        } else if is_exit {
                            Color32::from_rgb(34, 197, 94)
                        } else {
                            Color32::from_rgb(51, 65, 85)
                        };
                        let mut border_width = 1.0_f32;

                        // Movement Range Highlight
                        if self.command_mode == CommandMode::Move && sel_can_act && !is_obs {
                            if let Some(sp) = sel_pos {
                                if sp.manhattan_distance(tile) <= 3 && self.find_unit_at(tile).is_none() {
                                    tile_color = Color32::from_rgba_unmultiplied(14, 116, 144, 90);
                                    border_color = Color32::from_rgb(56, 189, 248);
                                    border_width = 1.5_f32;
                                }
                            }
                        }

                        // Attack Range Highlight
                        if (self.command_mode == CommandMode::Strike || self.command_mode == CommandMode::Skill) && sel_can_act {
                            if let Some(sp) = sel_pos {
                                if sp.manhattan_distance(tile) <= sel_range {
                                    tile_color = Color32::from_rgba_unmultiplied(153, 27, 27, 80);
                                    border_color = Color32::from_rgb(239, 68, 68);
                                    border_width = 1.5_f32;
                                }
                            }
                        }

                        // Hover Highlight
                        if self.hovered_tile == Some(tile) {
                            tile_color = Color32::from_rgba_unmultiplied(255, 255, 255, 40);
                            border_color = Color32::WHITE;
                            border_width = 2.0_f32;
                        }

                        painter.rect_filled(tile_rect, Rounding::same(3.0), tile_color);
                        painter.rect_stroke(tile_rect, Rounding::same(3.0), Stroke::new(border_width, border_color));

                        // Coordinate / Terrain watermark
                        let label = if is_obs {
                            "WALL".to_string()
                        } else if is_exit {
                            "EXIT".to_string()
                        } else {
                            format!("{},{}", x, y)
                        };
                        painter.text(
                            Pos2::new(tile_min.x + 3.0, tile_min.y + 3.0),
                            egui::Align2::LEFT_TOP,
                            label,
                            FontId::proportional(9.0),
                            if is_exit { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgba_unmultiplied(148, 163, 184, 50) },
                        );
                    }
                }

                // Draw Enemy Intent Tells (Dashed lines to their target heroes)
                for unit in &self.units {
                    if unit.team == Team::Enemies && unit.is_alive() {
                        if let Some(t_idx) = unit.intent_target_idx {
                            if self.units[t_idx].is_alive() {
                                let t_pos = self.units[t_idx].pos;
                                let e_pos = unit.pos;
                                let start = Pos2::new(grid_origin.x + (e_pos.x as f32 + 0.5) * cell_size, grid_origin.y + (e_pos.y as f32 + 0.5) * cell_size);
                                let end = Pos2::new(grid_origin.x + (t_pos.x as f32 + 0.5) * cell_size, grid_origin.y + (t_pos.y as f32 + 0.5) * cell_size);
                                painter.line_segment([start, end], Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(248, 113, 113, 90)));
                            }
                        }
                    }
                }

                // Draw Target Line if an enemy is hovered in attack mode
                if self.command_mode == CommandMode::Strike || self.command_mode == CommandMode::Skill {
                    if let (Some(s_pos), Some(h_idx)) = (sel_pos, self.hovered_unit_idx) {
                        if self.units[h_idx].team == Team::Enemies {
                            let e_pos = self.units[h_idx].pos;
                            let start = Pos2::new(grid_origin.x + (s_pos.x as f32 + 0.5) * cell_size, grid_origin.y + (s_pos.y as f32 + 0.5) * cell_size);
                            let end = Pos2::new(grid_origin.x + (e_pos.x as f32 + 0.5) * cell_size, grid_origin.y + (e_pos.y as f32 + 0.5) * cell_size);
                            painter.line_segment([start, end], Stroke::new(2.5_f32, Color32::from_rgb(244, 63, 94)));
                        }
                    }
                }

                // Draw Units
                for (idx, unit) in self.units.iter().enumerate() {
                    if !unit.is_alive() {
                        continue;
                    }

                    let tile_center = Pos2::new(
                        grid_origin.x + (unit.pos.x as f32 + 0.5) * cell_size,
                        grid_origin.y + (unit.pos.y as f32 + 0.5) * cell_size,
                    );

                    let token_radius = cell_size * 0.32;
                    let is_selected = self.selected_unit_idx == Some(idx);
                    let is_hovered = self.hovered_unit_idx == Some(idx);

                    // Selection / Pulse Ring
                    if is_selected {
                        painter.circle_stroke(tile_center, token_radius + 4.0, Stroke::new(2.5_f32, Color32::from_rgb(250, 204, 21)));
                    } else if is_hovered {
                        painter.circle_stroke(tile_center, token_radius + 2.0, Stroke::new(1.5_f32, Color32::WHITE));
                    }

                    // Token Body
                    let (fill_color, border_color) = if unit.team == Team::Allies {
                        (Color32::from_rgb(37, 99, 235), Color32::from_rgb(96, 165, 250))
                    } else {
                        (Color32::from_rgb(220, 38, 38), Color32::from_rgb(248, 113, 113))
                    };

                    painter.circle_filled(tile_center, token_radius, fill_color);
                    painter.circle_stroke(tile_center, token_radius, Stroke::new(2.0_f32, border_color));

                    // Unit Symbol
                    let symbol = match unit.id.as_str() {
                        "warrior" => "W",
                        "ranger" => "R",
                        "priest" => "P",
                        "goblin_brawler" => "B",
                        "goblin_skirmisher" => "S",
                        "goblin_archer" => "A",
                        "goblin_shaman" => "M",
                        "goblin_berserker" => "Z",
                        _ => "?",
                    };
                    painter.text(tile_center, egui::Align2::CENTER_CENTER, symbol, FontId::proportional(cell_size * 0.28), Color32::WHITE);

                    // Mini HP bar above unit
                    let hp_w = cell_size * 0.7;
                    let hp_h = 4.0;
                    let hp_x = tile_center.x - hp_w / 2.0;
                    let hp_y = tile_center.y - token_radius - 8.0;
                    let hp_bg = Rect::from_min_size(Pos2::new(hp_x, hp_y), Vec2::new(hp_w, hp_h));
                    let hp_pct = (unit.current_hp as f32 / unit.max_hp as f32).clamp(0.0, 1.0);
                    let hp_fg = Rect::from_min_size(Pos2::new(hp_x, hp_y), Vec2::new(hp_w * hp_pct, hp_h));

                    painter.rect_filled(hp_bg, Rounding::same(1.0), Color32::from_rgb(30, 41, 59));
                    painter.rect_filled(hp_fg, Rounding::same(1.0), if unit.team == Team::Allies { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(239, 68, 68) });

                    // Unit Name and Stance badge below unit
                    painter.text(
                        Pos2::new(tile_center.x, tile_center.y + token_radius + 3.0),
                        egui::Align2::CENTER_TOP,
                        format!("{} [{}]", unit.name, unit.stance.icon()),
                        FontId::proportional(10.0),
                        Color32::from_rgb(226, 232, 240),
                    );
                }

                // Draw Floating Texts
                for f in &self.floating_texts {
                    let elapsed = f.spawned.elapsed().as_secs_f32();
                    let progress = (elapsed / f.duration.as_secs_f32()).clamp(0.0, 1.0);
                    let alpha = ((1.0 - progress) * 255.0) as u8;

                    let base_x = grid_origin.x + (f.grid_pos.x as f32 + 0.5) * cell_size;
                    let base_y = grid_origin.y + (f.grid_pos.y as f32 + 0.3) * cell_size - (progress * 30.0);

                    let c = Color32::from_rgba_unmultiplied(f.color.r(), f.color.g(), f.color.b(), alpha);
                    painter.text(Pos2::new(base_x, base_y), egui::Align2::CENTER_CENTER, &f.text, FontId::proportional(16.0), c);
                }

                // Draw Combat Forecast Tooltip Box
                if let (Some(s_idx), Some(h_idx)) = (self.selected_unit_idx, self.hovered_unit_idx) {
                    let sel = &self.units[s_idx];
                    let target = &self.units[h_idx];

                    if sel.team == Team::Allies && target.team == Team::Enemies && sel.pos.manhattan_distance(target.pos) <= sel.range {
                        if let Some(hover_pos) = response.hover_pos() {
                            let box_pos = Pos2::new(hover_pos.x + 15.0, hover_pos.y - 20.0);
                            let box_rect = Rect::from_min_size(box_pos, Vec2::new(180.0, 105.0));

                            painter.rect_filled(box_rect, Rounding::same(6.0), Color32::from_rgb(15, 23, 42));
                            painter.rect_stroke(box_rect, Rounding::same(6.0), Stroke::new(1.5_f32, Color32::from_rgb(250, 204, 21)));

                            let mult = sel.stance.multiplier_vs(target.stance);
                            let est_dmg = ((sel.attack as f32 * mult) as i32 - target.armor).max(1);
                            let stance_info = if mult > 1.0 { "+25% (Advantage)" } else if mult < 1.0 { "-25% (Disadvantage)" } else { "Even" };
                            let est_counter = if target.pos.manhattan_distance(sel.pos) <= target.range {
                                format!("-{} DMG", (target.attack - sel.armor).max(1))
                            } else {
                                "None (Out of range)".to_string()
                            };

                            painter.text(Pos2::new(box_pos.x + 8.0, box_pos.y + 6.0), egui::Align2::LEFT_TOP, "⚔ COMBAT FORECAST", FontId::proportional(11.0), Color32::from_rgb(250, 204, 21));
                            painter.text(Pos2::new(box_pos.x + 8.0, box_pos.y + 24.0), egui::Align2::LEFT_TOP, format!("Target: {} [{}]", target.name, target.stance.name()), FontId::proportional(10.0), Color32::WHITE);
                            painter.text(Pos2::new(box_pos.x + 8.0, box_pos.y + 40.0), egui::Align2::LEFT_TOP, format!("Projected Dmg: {} DMG", est_dmg), FontId::proportional(11.0), Color32::from_rgb(34, 197, 94));
                            painter.text(Pos2::new(box_pos.x + 8.0, box_pos.y + 56.0), egui::Align2::LEFT_TOP, format!("Stance Matchup: {}", stance_info), FontId::proportional(9.0), Color32::from_rgb(250, 204, 21));
                            painter.text(Pos2::new(box_pos.x + 8.0, box_pos.y + 72.0), egui::Align2::LEFT_TOP, "Hit Probability: 95%", FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
                            painter.text(Pos2::new(box_pos.x + 8.0, box_pos.y + 88.0), egui::Align2::LEFT_TOP, format!("Counter: {}", est_counter), FontId::proportional(10.0), Color32::from_rgb(248, 113, 113));
                        }
                    }
                }

                // Handle Grid Clicking
                if response.clicked() {
                    if let Some(tile) = self.hovered_tile {
                        let clicked_unit_idx = self.find_unit_at(tile);

                        match self.command_mode {
                            CommandMode::Move => {
                                if let Some(s_idx) = self.selected_unit_idx {
                                    let s_pos = self.units[s_idx].pos;
                                    if s_pos.manhattan_distance(tile) <= 3 && clicked_unit_idx.is_none() && !self.is_obstacle(tile) {
                                        self.execute_move(s_idx, tile);
                                    }
                                }
                            }
                            CommandMode::Strike => {
                                if let (Some(s_idx), Some(t_idx)) = (self.selected_unit_idx, clicked_unit_idx) {
                                    if self.units[t_idx].team == Team::Enemies {
                                        let dist = self.units[s_idx].pos.manhattan_distance(tile);
                                        if dist <= self.units[s_idx].range {
                                            self.execute_attack(s_idx, t_idx);
                                        }
                                    }
                                }
                            }
                            CommandMode::Skill => {
                                if let (Some(s_idx), Some(t_idx)) = (self.selected_unit_idx, clicked_unit_idx) {
                                    if self.units[t_idx].team == Team::Enemies {
                                        let dist = self.units[s_idx].pos.manhattan_distance(tile);
                                        if dist <= self.units[s_idx].range + 1 {
                                            self.execute_skill(s_idx, Some(t_idx));
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Default selection mode: select clicked unit if it exists
                                if let Some(u_idx) = clicked_unit_idx {
                                    if self.units[u_idx].team == Team::Allies {
                                        self.selected_unit_idx = Some(u_idx);
                                        self.command_mode = CommandMode::None;
                                    }
                                }
                            }
                        }
                    }
                }
            });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 760.0])
            .with_min_inner_size([1024.0, 640.0])
            .with_title("Project Aegis - 2D Tactical Engine (Godot Parity GUI)"),
        ..Default::default()
    };

    eframe::run_native(
        "Project Aegis",
        native_options,
        Box::new(|cc| Ok(Box::new(AegisApp::new(cc)))),
    )
}
