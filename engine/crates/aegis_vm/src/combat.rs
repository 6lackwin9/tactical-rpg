use aegis_core::{DeterministicRng, IVec2, Signal, Variant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Team {
    Allies,
    Enemies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    pub id: String,
    pub name: String,
    pub team: Team,
    pub max_hp: i32,
    pub current_hp: i32,
    pub attack: i32,
    pub armor: i32,
    pub speed: i32,
    pub position: IVec2,
    pub is_withdrawn: bool,
}

impl Combatant {
    pub fn new(id: &str, name: &str, team: Team, max_hp: i32, attack: i32, armor: i32, speed: i32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            team,
            max_hp,
            current_hp: max_hp,
            attack,
            armor,
            speed,
            position: IVec2::NONE,
            is_withdrawn: false,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0 && !self.is_withdrawn
    }

    /// Reduces HP by amount after armor rating, minimum 1 damage.
    pub fn take_damage(&mut self, incoming_power: i32) -> i32 {
        let actual_damage = (incoming_power - self.armor).max(1);
        self.current_hp = (self.current_hp - actual_damage).max(0);
        actual_damage
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let restored = (self.max_hp - self.current_hp).min(amount);
        self.current_hp += restored;
        restored
    }
}

pub struct CombatGrid {
    pub width: i32,
    pub height: i32,
}

impl Default for CombatGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatGrid {
    pub const GRID_W: i32 = 12;
    pub const GRID_H: i32 = 9;
    pub const PARTY_FRONT_X: i32 = 5;
    pub const PARTY_BACK_X: i32 = 4;
    pub const ENEMY_FRONT_X: i32 = 7;
    pub const ENEMY_BACK_X: i32 = 8;

    pub fn new() -> Self {
        Self {
            width: Self::GRID_W,
            height: Self::GRID_H,
        }
    }

    pub fn is_valid_coord(&self, coord: IVec2) -> bool {
        coord.is_in_bounds(self.width, self.height)
    }

    pub fn distance(a: IVec2, b: IVec2) -> i32 {
        a.manhattan_distance(b)
    }
}

#[derive(Debug, Clone)]
pub enum CommandKind {
    Attack { target_idx: usize, power: i32 },
    Defend,
    Move { target_pos: IVec2 },
    Rest,
}

#[derive(Debug, Clone)]
pub struct ScheduledAction {
    pub actor_team: Team,
    pub actor_idx: usize,
    pub command: CommandKind,
    pub start_block: i32,
    pub block_cost: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleOutcome {
    Ongoing,
    Victory,
    Defeat,
}

pub struct CombatSession {
    pub grid: CombatGrid,
    pub allies: Vec<Combatant>,
    pub enemies: Vec<Combatant>,
    pub current_round: i32,
    pub max_rounds: i32,
    pub total_blocks: i32,
    pub rng: DeterministicRng,
    pub log: Vec<String>,

    pub on_log_emitted: Signal,
    pub on_combatant_defeated: Signal,
}

impl CombatSession {
    pub fn new(allies: Vec<Combatant>, enemies: Vec<Combatant>, rng_seed: u64) -> Self {
        let mut session = Self {
            grid: CombatGrid::new(),
            allies,
            enemies,
            current_round: 1,
            max_rounds: 10,
            total_blocks: 6,
            rng: DeterministicRng::new(rng_seed),
            log: Vec::new(),
            on_log_emitted: Signal::new(),
            on_combatant_defeated: Signal::new(),
        };
        session.deploy_units();
        session
    }

    fn deploy_units(&mut self) {
        let rows = [4, 3, 5, 2, 6, 1, 7];
        for (i, ally) in self.allies.iter_mut().enumerate() {
            let row = rows.get(i).copied().unwrap_or(4);
            ally.position = IVec2::new(CombatGrid::PARTY_FRONT_X, row);
        }
        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            let row = rows.get(i).copied().unwrap_or(4);
            enemy.position = IVec2::new(CombatGrid::ENEMY_FRONT_X, row);
        }
    }

    pub fn log_message(&mut self, msg: String) {
        self.on_log_emitted.emit(&[Variant::String(msg.clone())]);
        self.log.push(msg);
    }

    pub fn check_outcome(&self) -> BattleOutcome {
        let allies_alive = self.allies.iter().any(|a| a.is_alive());
        let enemies_alive = self.enemies.iter().any(|e| e.is_alive());

        if !allies_alive {
            BattleOutcome::Defeat
        } else if !enemies_alive {
            BattleOutcome::Victory
        } else {
            BattleOutcome::Ongoing
        }
    }

    /// Automatically resolves one full round of combat using basic tactical heuristics.
    pub fn resolve_round_autoplay(&mut self) -> BattleOutcome {
        self.log_message(format!("--- Round {} begins ---", self.current_round));

        // 1. Generate scheduled actions for all live combatants
        let mut actions: Vec<ScheduledAction> = Vec::new();

        // Allies target nearest enemy
        for (a_idx, ally) in self.allies.iter().enumerate() {
            if !ally.is_alive() {
                continue;
            }
            if let Some(target_idx) = self.find_first_live_enemy() {
                actions.push(ScheduledAction {
                    actor_team: Team::Allies,
                    actor_idx: a_idx,
                    command: CommandKind::Attack {
                        target_idx,
                        power: ally.attack,
                    },
                    start_block: 1,
                    block_cost: 2,
                });
            }
        }

        // Enemies target nearest ally
        for (e_idx, enemy) in self.enemies.iter().enumerate() {
            if !enemy.is_alive() {
                continue;
            }
            if let Some(target_idx) = self.find_first_live_ally() {
                actions.push(ScheduledAction {
                    actor_team: Team::Enemies,
                    actor_idx: e_idx,
                    command: CommandKind::Attack {
                        target_idx,
                        power: enemy.attack,
                    },
                    start_block: 1,
                    block_cost: 2,
                });
            }
        }

        // 2. Resolve actions block by block
        for block in 1..=self.total_blocks {
            for action in &actions {
                if action.start_block != block {
                    continue;
                }

                match action.command {
                    CommandKind::Attack { target_idx, power } => {
                        match action.actor_team {
                            Team::Allies => {
                                if !self.allies[action.actor_idx].is_alive() {
                                    continue;
                                }
                                let (dmg, target_name, target_alive, remaining_hp) = {
                                    if let Some(target) = self.enemies.get_mut(target_idx) {
                                        if target.is_alive() {
                                            let dmg = target.take_damage(power);
                                            (Some(dmg), target.name.clone(), target.is_alive(), target.current_hp)
                                        } else {
                                            (None, String::new(), false, 0)
                                        }
                                    } else {
                                        (None, String::new(), false, 0)
                                    }
                                };

                                if let Some(dmg) = dmg {
                                    let actor_name = self.allies[action.actor_idx].name.clone();
                                    self.log_message(format!(
                                        "[Block {}] {} attacks {} for {} dmg (HP remaining: {})",
                                        block, actor_name, target_name, dmg, remaining_hp
                                    ));

                                    if !target_alive {
                                        self.log_message(format!("{} has fallen!", target_name));
                                        self.on_combatant_defeated.emit(&[Variant::String(target_name)]);
                                    }
                                }
                            }
                            Team::Enemies => {
                                if !self.enemies[action.actor_idx].is_alive() {
                                    continue;
                                }
                                let (dmg, target_name, target_alive, remaining_hp) = {
                                    if let Some(target) = self.allies.get_mut(target_idx) {
                                        if target.is_alive() {
                                            let dmg = target.take_damage(power);
                                            (Some(dmg), target.name.clone(), target.is_alive(), target.current_hp)
                                        } else {
                                            (None, String::new(), false, 0)
                                        }
                                    } else {
                                        (None, String::new(), false, 0)
                                    }
                                };

                                if let Some(dmg) = dmg {
                                    let actor_name = self.enemies[action.actor_idx].name.clone();
                                    self.log_message(format!(
                                        "[Block {}] {} attacks {} for {} dmg (HP remaining: {})",
                                        block, actor_name, target_name, dmg, remaining_hp
                                    ));

                                    if !target_alive {
                                        self.log_message(format!("{} has fallen!", target_name));
                                        self.on_combatant_defeated.emit(&[Variant::String(target_name)]);
                                    }
                                }
                            }
                        }
                    }
                    CommandKind::Defend => {}
                    CommandKind::Move { .. } => {}
                    CommandKind::Rest => {}
                }

                // Check early termination
                let outcome = self.check_outcome();
                if outcome != BattleOutcome::Ongoing {
                    return outcome;
                }
            }
        }

        self.current_round += 1;
        self.check_outcome()
    }

    fn find_first_live_enemy(&self) -> Option<usize> {
        self.enemies.iter().position(|e| e.is_alive())
    }

    fn find_first_live_ally(&self) -> Option<usize> {
        self.allies.iter().position(|a| a.is_alive())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_simulation_victory() {
        let allies = vec![Combatant::new("warrior", "Warrior", Team::Allies, 60, 16, 4, 10)];
        let enemies = vec![Combatant::new("goblin", "Goblin", Team::Enemies, 25, 8, 1, 8)];

        let mut session = CombatSession::new(allies, enemies, 1337);
        assert_eq!(session.allies[0].position, IVec2::new(5, 4));
        assert_eq!(session.enemies[0].position, IVec2::new(7, 4));

        let mut outcome = BattleOutcome::Ongoing;
        while outcome == BattleOutcome::Ongoing && session.current_round <= 5 {
            outcome = session.resolve_round_autoplay();
        }

        assert_eq!(outcome, BattleOutcome::Victory);
        assert!(session.allies[0].is_alive());
        assert!(!session.enemies[0].is_alive());
    }
}
