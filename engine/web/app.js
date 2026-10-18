/**
 * PROJECT AEGIS - High-Fidelity 2D Tactical Web Runtime
 * Interactive Player GUI with Pathfinding, Command Dock, Combat Forecast & AI.
 */

// --- 1. Procedural Web Audio Synthesizer ---
class TacticalAudio {
  constructor() {
    this.ctx = null;
    this.muted = false;
  }

  init() {
    if (!this.ctx) {
      const AudioCtx = window.AudioContext || window.webkitAudioContext;
      if (AudioCtx) this.ctx = new AudioCtx();
    }
  }

  playStep() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'triangle';
    osc.frequency.setValueAtTime(140, this.ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(80, this.ctx.currentTime + 0.08);
    gain.gain.setValueAtTime(0.12, this.ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, this.ctx.currentTime + 0.08);
    osc.connect(gain);
    gain.connect(this.ctx.destination);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.08);
  }

  playStrike() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sawtooth';
    osc.frequency.setValueAtTime(220, this.ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(45, this.ctx.currentTime + 0.14);
    gain.gain.setValueAtTime(0.22, this.ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, this.ctx.currentTime + 0.14);
    osc.connect(gain);
    gain.connect(this.ctx.destination);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.14);
  }

  playBow() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(620, this.ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(260, this.ctx.currentTime + 0.18);
    gain.gain.setValueAtTime(0.16, this.ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, this.ctx.currentTime + 0.18);
    osc.connect(gain);
    gain.connect(this.ctx.destination);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.18);
  }

  playGuard() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(320, this.ctx.currentTime);
    osc.frequency.linearRampToValueAtTime(440, this.ctx.currentTime + 0.2);
    gain.gain.setValueAtTime(0.18, this.ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, this.ctx.currentTime + 0.2);
    osc.connect(gain);
    gain.connect(this.ctx.destination);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.2);
  }

  playDefeat() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'triangle';
    osc.frequency.setValueAtTime(130, this.ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(30, this.ctx.currentTime + 0.5);
    gain.gain.setValueAtTime(0.3, this.ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, this.ctx.currentTime + 0.5);
    osc.connect(gain);
    gain.connect(this.ctx.destination);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.5);
  }

  playVictory() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const notes = [261.6, 329.6, 392.0, 523.2];
    notes.forEach((freq, i) => {
      const osc = this.ctx.createOscillator();
      const gain = this.ctx.createGain();
      osc.type = 'sine';
      osc.frequency.value = freq;
      gain.gain.setValueAtTime(0.18, this.ctx.currentTime + i * 0.1);
      gain.gain.exponentialRampToValueAtTime(0.01, this.ctx.currentTime + i * 0.1 + 0.35);
      osc.connect(gain);
      gain.connect(this.ctx.destination);
      osc.start(this.ctx.currentTime + i * 0.1);
      osc.stop(this.ctx.currentTime + i * 0.1 + 0.35);
    });
  }
}

// --- 2. Data Catalogs ---
const CATALOG = {
  encounters: {
    goblin_skirmish: {
      name: "Goblin Skirmish",
      zone: "Ruined Courtyard",
      max_rounds: 6,
      party: ["warrior", "ranger", "priest"],
      enemies: ["goblin_brawler", "goblin_skirmisher", "goblin_archer", "goblin_shaman", "goblin_berserker"]
    },
    veterans_drill: {
      name: "Veterans' Drill",
      zone: "Stone Arena",
      max_rounds: 6,
      party: ["dojo_warrior", "dojo_ranger"],
      enemies: ["goblin_brawler", "goblin_archer", "goblin_archer_2"]
    },
    goblin_ambush: {
      name: "Goblin Ambush",
      zone: "Forest Clearing",
      max_rounds: 6,
      party: ["warrior", "ranger"],
      enemies: ["enemy_brawler", "fleeing_scout"]
    }
  },
  units: {
    warrior: {
      name: "Warrior", class_id: "Warrior", max_hp: 242, attack: 41, armor: 18, speed: 10,
      badge: "W", role: "Vanguard", pref: "Nearest Foe",
      skill_name: "Shield Bash", skill_cost: 5, skill_range: 1, skill_power: 48
    },
    dojo_warrior: {
      name: "Veteran Warrior", class_id: "Warrior", max_hp: 242, attack: 41, armor: 18, speed: 10,
      badge: "W", role: "Vanguard", pref: "Nearest Foe",
      skill_name: "Shield Bash", skill_cost: 5, skill_range: 1, skill_power: 48
    },
    ranger: {
      name: "Ranger", class_id: "Ranger", max_hp: 158, attack: 35, armor: 6, speed: 12,
      badge: "R", role: "Marksman", pref: "Lowest HP Foe",
      skill_name: "Aimed Shot", skill_cost: 5, skill_range: 4, skill_power: 52
    },
    dojo_ranger: {
      name: "Veteran Ranger", class_id: "Ranger", max_hp: 158, attack: 35, armor: 6, speed: 12,
      badge: "R", role: "Marksman", pref: "Lowest HP Foe",
      skill_name: "Aimed Shot", skill_cost: 5, skill_range: 4, skill_power: 52
    },
    priest: {
      name: "Priest", class_id: "Priest", max_hp: 176, attack: 34, armor: 5, speed: 9,
      badge: "P", role: "Support", pref: "Weakest Foe",
      skill_name: "Smite II", skill_cost: 5, skill_range: 3, skill_power: 46
    },
    goblin_brawler: {
      name: "Goblin Brawler", class_id: "Brawler", max_hp: 108, attack: 31, armor: 9, speed: 8,
      badge: "B", role: "Bruiser", pref: "Frontline", skill_name: "Body Blow", skill_range: 1
    },
    enemy_brawler: {
      name: "Goblin Brawler", class_id: "Brawler", max_hp: 108, attack: 31, armor: 9, speed: 8,
      badge: "B", role: "Bruiser", pref: "Frontline", skill_name: "Body Blow", skill_range: 1
    },
    goblin_skirmisher: {
      name: "Goblin Skirmisher", class_id: "Rogue", max_hp: 65, attack: 32, armor: 7, speed: 14,
      badge: "S", role: "Flanker", pref: "Squishy Target", skill_name: "Twin Daggers", skill_range: 1
    },
    fleeing_scout: {
      name: "Goblin Scout", class_id: "Rogue", max_hp: 65, attack: 32, armor: 7, speed: 14,
      badge: "S", role: "Scout", pref: "Escape / Flank", skill_name: "Shiv", skill_range: 1
    },
    goblin_archer: {
      name: "Goblin Archer", class_id: "Ranger", max_hp: 71, attack: 35, armor: 7, speed: 13,
      badge: "A", role: "Ranged", pref: "Warrior", skill_name: "Pinning Shot", skill_range: 3
    },
    goblin_archer_2: {
      name: "Goblin Archer", class_id: "Ranger", max_hp: 71, attack: 35, armor: 7, speed: 13,
      badge: "A", role: "Ranged", pref: "Ranger", skill_name: "Pinning Shot", skill_range: 3
    },
    goblin_shaman: {
      name: "Goblin Shaman", class_id: "Mage", max_hp: 77, attack: 25, armor: 6, speed: 7,
      badge: "M", role: "Caster", pref: "Weakest Hero", skill_name: "Cinder Burst", skill_range: 3
    },
    goblin_berserker: {
      name: "Goblin Berserker", class_id: "Berserker", max_hp: 108, attack: 31, armor: 5, speed: 11,
      badge: "Z", role: "Berserker", pref: "Highest Threat", skill_name: "Rage Flurry", skill_range: 1
    }
  }
};

// --- 3. Combat Simulation Engine ---
class CombatSession {
  constructor(encounterKey = "goblin_skirmish") {
    this.GRID_W = 12;
    this.GRID_H = 9;
    this.PARTY_COL = 5;
    this.ENEMY_COL = 7;
    this.encounterKey = encounterKey;
    this.encounter = CATALOG.encounters[encounterKey];

    this.round = 1;
    this.max_rounds = this.encounter.max_rounds;
    this.outcome = "ongoing";

    this.allies = [];
    this.enemies = [];
    this.log = [];

    this.initCombatants();
  }

  initCombatants() {
    const allyRows = [4, 3, 5, 2, 6];
    this.allies = this.encounter.party.map((id, index) => {
      const proto = CATALOG.units[id] || CATALOG.units.warrior;
      const startY = allyRows[index % allyRows.length];
      return {
        id,
        name: proto.name,
        class_id: proto.class_id,
        badge: proto.badge,
        team: "allies",
        max_hp: proto.max_hp,
        hp: proto.max_hp,
        attack: proto.attack,
        armor: proto.armor,
        speed: proto.speed,
        role: proto.role,
        pref: proto.pref,
        skill_name: proto.skill_name || "Strike",
        skill_power: proto.skill_power || (proto.attack + 10),
        skill_range: proto.skill_range || 1,
        ap: 12,
        max_ap: 12,
        x: this.PARTY_COL,
        y: startY,
        renderX: this.PARTY_COL,
        renderY: startY,
        alive: true,
        guarded: false,
        intent: "Ready"
      };
    });

    const enemyRows = [4, 3, 5, 2, 6];
    this.enemies = this.encounter.enemies.map((id, index) => {
      const proto = CATALOG.units[id] || CATALOG.units.goblin_brawler;
      const startY = enemyRows[index % enemyRows.length];
      return {
        id,
        name: proto.name,
        class_id: proto.class_id,
        badge: proto.badge,
        team: "enemies",
        max_hp: proto.max_hp,
        hp: proto.max_hp,
        attack: proto.attack,
        armor: proto.armor,
        speed: proto.speed,
        role: proto.role,
        pref: proto.pref,
        skill_name: proto.skill_name || "Bite",
        skill_range: proto.skill_range || 1,
        x: this.ENEMY_COL,
        y: startY,
        renderX: this.ENEMY_COL,
        renderY: startY,
        alive: true,
        guarded: false,
        intent: "Advance & Attack"
      };
    });
  }

  getUnitAt(x, y) {
    return this.allies.find(a => a.x === x && a.y === y && a.alive) ||
           this.enemies.find(e => e.x === x && e.y === y && e.alive) || null;
  }

  isTileOccupied(x, y) {
    return this.getUnitAt(x, y) !== null;
  }

  calculateDistance(u1, u2) {
    return Math.abs(u1.x - u2.x) + Math.abs(u1.y - u2.y);
  }

  // Find reachable tiles for unit (Manhattan range)
  getReachableTiles(unit) {
    const range = Math.min(4, Math.floor(unit.speed / 3) + 1);
    const reachable = [];

    for (let dy = -range; dy <= range; dy++) {
      for (let dx = -range; dx <= range; dx++) {
        const nx = unit.x + dx;
        const ny = unit.y + dy;
        const dist = Math.abs(dx) + Math.abs(dy);

        if (dist > 0 && dist <= range && nx >= 0 && nx < this.GRID_W && ny >= 0 && ny < this.GRID_H) {
          if (!this.isTileOccupied(nx, ny)) {
            reachable.push({ x: nx, y: ny, cost: dist });
          }
        }
      }
    }
    return reachable;
  }

  // Find valid enemy targets in attack reach
  getValidTargets(unit, range = 1) {
    const targets = [];
    const foes = unit.team === "allies" ? this.enemies : this.allies;

    for (const foe of foes) {
      if (!foe.alive) continue;
      const dist = Math.abs(unit.x - foe.x) + Math.abs(unit.y - foe.y);
      if (dist <= range) {
        targets.push(foe);
      }
    }
    return targets;
  }

  // Execute Player Action: Move
  moveUnit(unit, targetX, targetY, audio) {
    const dist = Math.abs(unit.x - targetX) + Math.abs(unit.y - targetY);
    if (dist === 0 || this.isTileOccupied(targetX, targetY)) return false;

    const oldX = unit.x;
    const oldY = unit.y;
    unit.x = targetX;
    unit.y = targetY;
    unit.ap = Math.max(0, unit.ap - dist);

    audio.playStep();
    this.logMessage(`[Move] ${unit.name} stepped from (${oldX}, ${oldY}) to (${targetX}, ${targetY}) [-${dist} AP]`, "ally-action");
    return true;
  }

  // Execute Player Action: Attack or Skill
  attackTarget(attacker, defender, isSkill, audio) {
    if (!attacker.alive || !defender.alive) return false;

    const basePower = isSkill ? attacker.skill_power : attacker.attack;
    const mitigation = defender.guarded ? 0.7 : 1.0;
    const rawDmg = Math.max(1, basePower - defender.armor);
    const finalDmg = Math.max(1, Math.floor(rawDmg * mitigation));

    defender.hp = Math.max(0, defender.hp - finalDmg);

    if (attacker.badge === "R") audio.playBow(); else audio.playStrike();

    // Damage floating text & particles
    const dmgLabel = defender.guarded ? `-${finalDmg} (GUARDED)` : `-${finalDmg}`;
    spawnFloatingText(defender.x, defender.y, dmgLabel, defender.guarded ? "#38bdf8" : "#fbbf24");
    spawnParticles(defender.x, defender.y, "#fbbf24");

    const actionName = isSkill ? attacker.skill_name : "Strike";
    this.logMessage(`[Action] ${attacker.name} uses ${actionName} on ${defender.name} for ${finalDmg} DMG (${defender.hp} HP left)`, "ally-action");

    if (defender.hp === 0) {
      defender.alive = false;
      audio.playDefeat();
      this.logMessage(`*** [DEFEAT] ${defender.name} was slain in battle! ***`, "defeat-banner");
    }

    attacker.ap = Math.max(0, attacker.ap - (isSkill ? 5 : 4));
    this.checkVerdict(audio);
    return true;
  }

  // Raise Guard
  guardUnit(unit, audio) {
    unit.guarded = true;
    unit.ap = Math.max(0, unit.ap - 2);
    audio.playGuard();
    spawnFloatingText(unit.x, unit.y, "GUARD UP (30%)", "#38bdf8");
    this.logMessage(`[Guard] ${unit.name} raises guard stance (mitigates 30% incoming damage)`, "ally-action");
  }

  // Execute Enemy AI turn for a single enemy
  runEnemyAI(enemy, audio) {
    if (!enemy.alive) return;

    // Reset guard each round
    enemy.guarded = false;

    // 1. Target evaluation: find best target
    let target = null;
    let bestScore = -999;

    for (const hero of this.allies) {
      if (!hero.alive) continue;
      const dist = this.calculateDistance(enemy, hero);
      let score = -dist; // Closer is better

      if (enemy.pref === "Squishy Target" || enemy.pref === "Lowest HP Foe") {
        score += (hero.max_hp - hero.hp);
      } else if (enemy.pref === "Highest Threat") {
        score += hero.attack;
      }
      if (score > bestScore) {
        bestScore = score;
        target = hero;
      }
    }

    if (!target) return;

    const reach = enemy.skill_range || 1;
    let dist = this.calculateDistance(enemy, target);

    // 2. Movement logic: advance towards target if not in reach
    if (dist > reach) {
      const stepX = Math.sign(target.x - enemy.x);
      const stepY = Math.sign(target.y - enemy.y);

      // Try stepping horizontally or vertically
      let nx = enemy.x + stepX;
      let ny = enemy.y;

      if (this.isTileOccupied(nx, ny) || nx < 0 || nx >= this.GRID_W) {
        nx = enemy.x;
        ny = enemy.y + stepY;
      }

      if (!this.isTileOccupied(nx, ny) && nx >= 0 && nx < this.GRID_W && ny >= 0 && ny < this.GRID_H) {
        const ox = enemy.x;
        const oy = enemy.y;
        enemy.x = nx;
        enemy.y = ny;
        audio.playStep();
        this.logMessage(`[Enemy Move] ${enemy.name} marched from (${ox}, ${oy}) to (${nx}, ${ny})`, "enemy-action");
      }
    }

    // 3. Attack if in reach after movement
    dist = this.calculateDistance(enemy, target);
    if (dist <= reach) {
      const rawDmg = Math.max(1, enemy.attack - target.armor);
      const mitigation = target.guarded ? 0.7 : 1.0;
      const finalDmg = Math.max(1, Math.floor(rawDmg * mitigation));

      target.hp = Math.max(0, target.hp - finalDmg);

      if (enemy.badge === "A") audio.playBow(); else audio.playStrike();

      spawnFloatingText(target.x, target.y, target.guarded ? `-${finalDmg} (GUARDED)` : `-${finalDmg}`, target.guarded ? "#38bdf8" : "#f43f5e");
      spawnParticles(target.x, target.y, "#f43f5e");

      this.logMessage(`[Enemy Action] ${enemy.name} attacks ${target.name} for ${finalDmg} DMG (${target.hp} HP remaining)`, "enemy-action");

      if (target.hp === 0) {
        target.alive = false;
        audio.playDefeat();
        this.logMessage(`*** [DEFEAT] ${target.name} has fallen! ***`, "defeat-banner");
      }
    }
  }

  // Advance / Resolve Round: Enemies take turns, reset AP for new round
  resolveRound(audio) {
    if (this.outcome !== "ongoing" || this.round > this.max_rounds) return;

    this.logMessage(`--- Hostile Turn: Round ${this.round} ---`, "round-banner");

    // All living enemies act
    for (const enemy of this.enemies) {
      this.runEnemyAI(enemy, audio);
    }

    this.checkVerdict(audio);

    if (this.outcome === "ongoing") {
      this.round += 1;
      // Refresh party AP and reset guard
      for (const ally of this.allies) {
        ally.ap = ally.max_ap;
        ally.guarded = false;
      }
      this.logMessage(`>>> Round ${this.round} / ${this.max_rounds} planning begins. Action points refreshed!`, "round-banner");
    }
  }

  checkVerdict(audio) {
    const alliesAlive = this.allies.some(a => a.alive);
    const enemiesAlive = this.enemies.some(e => e.alive);

    if (!alliesAlive) {
      this.outcome = "defeat";
      this.logMessage("=== DEFEAT! All party members have fallen. ===", "defeat-banner");
    } else if (!enemiesAlive) {
      this.outcome = "victory";
      audio.playVictory();
      this.logMessage("=== VICTORY! The party cleared the encounter! ===", "round-banner");
    } else if (this.round >= this.max_rounds) {
      this.outcome = "timeout";
      this.logMessage("=== DRAW / TIMEOUT: Maximum rounds reached. ===", "round-banner");
    }
  }

  logMessage(text, type = "normal") {
    this.log.push({ text, type, time: new Date().toLocaleTimeString().slice(3, 8) });
  }
}

// --- 4. Particle & Floating Text Systems ---
const floatingTexts = [];
const particles = [];

function spawnFloatingText(gridX, gridY, text, color) {
  floatingTexts.push({
    gridX,
    gridY,
    text,
    color,
    yOffset: 0,
    alpha: 1.0,
    life: 50
  });
}

function spawnParticles(gridX, gridY, color) {
  for (let i = 0; i < 10; i++) {
    const angle = (Math.PI * 2 * i) / 10 + Math.random() * 0.4;
    const speed = 1.8 + Math.random() * 2.5;
    particles.push({
      gridX,
      gridY,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed,
      x: 0,
      y: 0,
      color,
      alpha: 1.0,
      life: 25
    });
  }
}

// --- 5. Application UI & Canvas Controller ---
class AegisApp {
  constructor() {
    this.audio = new TacticalAudio();
    this.session = new CombatSession("goblin_skirmish");
    this.canvas = document.getElementById("battlefield-canvas");
    this.ctx = this.canvas.getContext("2d");

    this.selectedUnit = this.session.allies[0];
    this.activeCommand = "none"; // 'none' | 'move' | 'attack' | 'skill' | 'guard'
    this.hoverTile = { x: -1, y: -1 };
    this.hoverUnit = null;
    this.autoplayInterval = null;

    this.tileW = 66;
    this.tileH = 56;
    this.originX = 24;
    this.originY = 36;

    this.initEventListeners();
    this.updateUI();
    this.renderLoop();
  }

  initEventListeners() {
    // 1. Top Buttons
    document.getElementById("btn-advance").addEventListener("click", () => {
      this.session.resolveRound(this.audio);
      this.activeCommand = "none";
      this.updateUI();
    });

    const autoplayBtn = document.getElementById("btn-autoplay");
    autoplayBtn.addEventListener("click", () => {
      if (this.autoplayInterval) {
        clearInterval(this.autoplayInterval);
        this.autoplayInterval = null;
        autoplayBtn.innerHTML = '<span class="btn-icon">▶</span> Auto-Play';
      } else {
        autoplayBtn.innerHTML = '<span class="btn-icon">⏸</span> Pause Auto';
        this.autoplayInterval = setInterval(() => {
          if (this.session.outcome !== "ongoing") {
            clearInterval(this.autoplayInterval);
            this.autoplayInterval = null;
            autoplayBtn.innerHTML = '<span class="btn-icon">▶</span> Auto-Play';
            return;
          }
          this.session.resolveRound(this.audio);
          this.updateUI();
        }, 700);
      }
    });

    document.getElementById("btn-reset").addEventListener("click", () => {
      const key = document.getElementById("encounter-select").value;
      this.session = new CombatSession(key);
      this.selectedUnit = this.session.allies[0];
      this.activeCommand = "none";
      if (this.autoplayInterval) {
        clearInterval(this.autoplayInterval);
        this.autoplayInterval = null;
        autoplayBtn.innerHTML = '<span class="btn-icon">▶</span> Auto-Play';
      }
      this.updateUI();
    });

    document.getElementById("btn-save").addEventListener("click", () => {
      const data = {
        encounter: this.session.encounterKey,
        round: this.session.round,
        outcome: this.session.outcome,
        allies: this.session.allies.map(a => ({ id: a.id, x: a.x, y: a.y, hp: a.hp, alive: a.alive })),
        enemies: this.session.enemies.map(e => ({ id: e.id, x: e.x, y: e.y, hp: e.hp, alive: e.alive }))
      };
      localStorage.setItem("aegis_slot_0", JSON.stringify(data));
      this.session.logMessage("[Save] State preserved to user://saves/slot_0.json", "round-banner");
      this.updateUI();
    });

    const audioBtn = document.getElementById("btn-audio");
    audioBtn.addEventListener("click", () => {
      this.audio.muted = !this.audio.muted;
      audioBtn.textContent = this.audio.muted ? "🔇" : "🔊";
    });

    document.getElementById("encounter-select").addEventListener("change", (e) => {
      this.session = new CombatSession(e.target.value);
      this.selectedUnit = this.session.allies[0];
      this.activeCommand = "none";
      this.updateUI();
    });

    document.getElementById("btn-clear-log").addEventListener("click", () => {
      this.session.log = [];
      this.updateUI();
    });

    // 2. Command Dock Buttons
    const cmdButtons = document.querySelectorAll(".cmd-btn");
    cmdButtons.forEach(btn => {
      btn.addEventListener("click", () => {
        const cmd = btn.getAttribute("data-cmd");
        if (!this.selectedUnit || this.selectedUnit.team !== "allies" || !this.selectedUnit.alive) return;

        if (cmd === "guard") {
          this.session.guardUnit(this.selectedUnit, this.audio);
          this.activeCommand = "none";
          this.updateUI();
        } else if (cmd === "hold") {
          this.session.logMessage(`[Hold] ${this.selectedUnit.name} holds position.`, "ally-action");
          this.activeCommand = "none";
          this.updateUI();
        } else {
          // Toggle command
          this.activeCommand = (this.activeCommand === cmd) ? "none" : cmd;
          this.updateUI();
        }
      });
    });

    // Keyboard Hotkeys
    window.addEventListener("keydown", (e) => {
      if (e.code === "Space") {
        e.preventDefault();
        this.session.resolveRound(this.audio);
        this.activeCommand = "none";
        this.updateUI();
      } else if (e.key === "1") {
        this.activeCommand = "move";
        this.updateUI();
      } else if (e.key === "2") {
        this.activeCommand = "attack";
        this.updateUI();
      } else if (e.key === "3") {
        this.activeCommand = "skill";
        this.updateUI();
      } else if (e.key === "4") {
        if (this.selectedUnit && this.selectedUnit.team === "allies") {
          this.session.guardUnit(this.selectedUnit, this.audio);
          this.activeCommand = "none";
          this.updateUI();
        }
      } else if (e.key === "Escape") {
        this.activeCommand = "none";
        this.updateUI();
      }
    });

    // 3. Canvas Mouse Interaction
    this.canvas.addEventListener("mousemove", (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const gx = Math.floor((mouseX - this.originX) / this.tileW);
      const gy = Math.floor((mouseY - this.originY) / this.tileH);

      const tooltip = document.getElementById("unit-tooltip");
      const forecastBanner = document.getElementById("forecast-banner");

      if (gx >= 0 && gx < this.session.GRID_W && gy >= 0 && gy < this.session.GRID_H) {
        this.hoverTile = { x: gx, y: gy };
        document.getElementById("cursor-coords").textContent = `Tile: [${gx}, ${gy}]`;

        const u = this.session.getUnitAt(gx, gy);
        this.hoverUnit = u;

        if (u) {
          tooltip.classList.remove("hidden");
          tooltip.style.left = `${mouseX}px`;
          tooltip.style.top = `${mouseY}px`;
          document.getElementById("tooltip-name").textContent = u.name;
          document.getElementById("tooltip-hp").textContent = `HP: ${u.hp} / ${u.max_hp}`;
          document.getElementById("tooltip-stats").textContent = `ATK ${u.attack} | ARM ${u.armor} | SPD ${u.speed}`;
        } else {
          tooltip.classList.add("hidden");
        }

        // Combat Forecast when targeting an enemy
        if ((this.activeCommand === "attack" || this.activeCommand === "skill") &&
            this.selectedUnit && this.selectedUnit.team === "allies" &&
            u && u.team === "enemies") {
          const isSkill = (this.activeCommand === "skill");
          const basePower = isSkill ? this.selectedUnit.skill_power : this.selectedUnit.attack;
          const predictedDmg = Math.max(1, basePower - u.armor);
          const postHp = Math.max(0, u.hp - predictedDmg);

          forecastBanner.classList.remove("hidden");
          document.getElementById("forecast-atk-name").textContent = `${this.selectedUnit.name} (${isSkill ? this.selectedUnit.skill_name : "Strike"})`;
          document.getElementById("forecast-atk-dmg").textContent = `${predictedDmg} DMG`;
          document.getElementById("forecast-def-name").textContent = u.name;
          document.getElementById("forecast-def-hp").textContent = `${postHp} / ${u.max_hp} HP (-${predictedDmg})`;
        } else {
          forecastBanner.classList.add("hidden");
        }
      } else {
        this.hoverTile = { x: -1, y: -1 };
        this.hoverUnit = null;
        tooltip.classList.add("hidden");
        forecastBanner.classList.add("hidden");
        document.getElementById("cursor-coords").textContent = `Tile: [—, —]`;
      }
    });

    this.canvas.addEventListener("mouseleave", () => {
      this.hoverTile = { x: -1, y: -1 };
      this.hoverUnit = null;
      document.getElementById("unit-tooltip").classList.add("hidden");
      document.getElementById("forecast-banner").classList.add("hidden");
    });

    this.canvas.addEventListener("click", () => {
      const { x, y } = this.hoverTile;
      if (x < 0 || y < 0) return;

      const clickedUnit = this.session.getUnitAt(x, y);

      // Handle active command execution
      if (this.selectedUnit && this.selectedUnit.team === "allies" && this.selectedUnit.alive) {
        if (this.activeCommand === "move") {
          const reachable = this.session.getReachableTiles(this.selectedUnit);
          const isValidMove = reachable.some(t => t.x === x && t.y === y);
          if (isValidMove) {
            this.session.moveUnit(this.selectedUnit, x, y, this.audio);
            this.activeCommand = "none";
            this.updateUI();
            return;
          }
        } else if (this.activeCommand === "attack" || this.activeCommand === "skill") {
          const isSkill = (this.activeCommand === "skill");
          const range = isSkill ? this.selectedUnit.skill_range : 1;
          const validTargets = this.session.getValidTargets(this.selectedUnit, range);

          if (clickedUnit && clickedUnit.team === "enemies" && validTargets.includes(clickedUnit)) {
            this.session.attackTarget(this.selectedUnit, clickedUnit, isSkill, this.audio);
            this.activeCommand = "none";
            this.updateUI();
            return;
          }
        }
      }

      // Default click: Unit Selection
      if (clickedUnit) {
        this.selectedUnit = clickedUnit;
        this.activeCommand = "none";
        this.updateUI();
      }
    });
  }

  updateUI() {
    // 1. Status Pill
    const roundDisp = document.getElementById("round-display");
    roundDisp.textContent = `${this.session.round} / ${this.session.max_rounds}`;

    const verdictDisp = document.getElementById("verdict-display");
    verdictDisp.className = `pill-status ${this.session.outcome}`;
    verdictDisp.textContent = this.session.outcome.toUpperCase();

    // 2. Arena Title & Active Command Badge
    document.getElementById("arena-name").textContent =
      `World: Ashlands • Zone: ${this.session.encounter.zone}`;

    const modeBadge = document.getElementById("active-mode-badge");
    modeBadge.textContent = `MODE: ${this.activeCommand.toUpperCase()}`;

    // 3. Command Dock Update
    const dockHeroName = document.getElementById("dock-hero-name");
    const dockAp = document.getElementById("dock-ap");
    const cmdButtons = document.querySelectorAll(".cmd-btn");

    if (this.selectedUnit && this.selectedUnit.team === "allies") {
      dockHeroName.textContent = this.selectedUnit.name;
      dockAp.textContent = `AP: ${this.selectedUnit.ap} / ${this.selectedUnit.max_ap}`;
      document.getElementById("cmd-skill-label").textContent = this.selectedUnit.skill_name;

      cmdButtons.forEach(btn => {
        btn.disabled = false;
        const cmd = btn.getAttribute("data-cmd");
        if (this.activeCommand === cmd) {
          btn.classList.add("active");
        } else {
          btn.classList.remove("active");
        }
      });
    } else {
      dockHeroName.textContent = this.selectedUnit ? `${this.selectedUnit.name} (Enemy)` : "None";
      dockAp.textContent = "AP: —";
      cmdButtons.forEach(btn => {
        btn.disabled = true;
        btn.classList.remove("active");
      });
    }

    // 4. Allies Roster List
    const alliesContainer = document.getElementById("allies-roster");
    alliesContainer.innerHTML = "";
    let liveAllies = 0;

    this.session.allies.forEach(ally => {
      if (ally.alive) liveAllies++;
      const card = document.createElement("div");
      const isSelected = this.selectedUnit === ally;
      card.className = `unit-card ${isSelected ? 'selected' : ''} ${!ally.alive ? 'defeated' : ''}`;

      const ratio = Math.max(0, ally.hp / ally.max_hp);
      const hpColorClass = ratio > 0.5 ? '' : (ratio > 0.25 ? 'warn' : 'danger');

      card.innerHTML = `
        <div class="card-top">
          <div class="unit-identity">
            <div class="unit-badge-icon">${ally.badge}</div>
            <div>
              <div class="unit-name">${ally.name}</div>
              <div class="unit-class-badge">${ally.class_id} • Pos (${ally.x}, ${ally.y})</div>
            </div>
          </div>
        </div>
        <div class="hp-bar-wrapper">
          <div class="hp-meta">
            <span class="hp-label">HP</span>
            <span class="hp-numbers">${ally.hp} / ${ally.max_hp}</span>
          </div>
          <div class="hp-progress">
            <div class="hp-progress-fill ${hpColorClass}" style="width: ${ratio * 100}%"></div>
          </div>
        </div>
        <div class="card-stats">
          <span>ATK ${ally.attack}</span>
          <span>ARM ${ally.armor}</span>
          <span>SPD ${ally.speed}</span>
          <span style="color:#fbbf24; margin-left:auto;">AP ${ally.ap}</span>
        </div>
      `;

      card.addEventListener("click", () => {
        this.selectedUnit = ally;
        this.activeCommand = "none";
        this.updateUI();
      });

      alliesContainer.appendChild(card);
    });

    document.getElementById("party-count").textContent = `${liveAllies} ALIVE`;

    // 5. Enemies Roster List
    const enemiesContainer = document.getElementById("enemies-roster");
    enemiesContainer.innerHTML = "";
    let liveEnemies = 0;

    this.session.enemies.forEach(enemy => {
      if (enemy.alive) liveEnemies++;
      const card = document.createElement("div");
      const isSelected = this.selectedUnit === enemy;
      card.className = `unit-card enemy-card ${isSelected ? 'selected' : ''} ${!enemy.alive ? 'defeated' : ''}`;

      const ratio = Math.max(0, enemy.hp / enemy.max_hp);

      card.innerHTML = `
        <div class="card-top">
          <div class="unit-identity">
            <div class="unit-badge-icon">${enemy.badge}</div>
            <div>
              <div class="unit-name">${enemy.name}</div>
              <div class="unit-class-badge">${enemy.class_id} • Pos (${enemy.x}, ${enemy.y})</div>
            </div>
          </div>
        </div>
        <div class="hp-bar-wrapper">
          <div class="hp-meta">
            <span class="hp-label">HP</span>
            <span class="hp-numbers">${enemy.hp} / ${enemy.max_hp}</span>
          </div>
          <div class="hp-progress">
            <div class="hp-progress-fill danger" style="width: ${ratio * 100}%"></div>
          </div>
        </div>
      `;

      card.addEventListener("click", () => {
        this.selectedUnit = enemy;
        this.activeCommand = "none";
        this.updateUI();
      });

      enemiesContainer.appendChild(card);
    });

    document.getElementById("enemy-count").textContent = `${liveEnemies} ALIVE`;

    // 6. Unit Inspector
    if (this.selectedUnit) {
      const u = this.selectedUnit;
      document.getElementById("inspect-avatar").textContent = u.team === "allies" ? "🛡️" : "💀";
      document.getElementById("inspect-name").textContent = u.name;
      document.getElementById("inspect-class").textContent = `${u.class_id} (${u.team.toUpperCase()})`;
      document.getElementById("inspect-hp").textContent = `${u.hp} / ${u.max_hp}`;
      document.getElementById("inspect-atk").textContent = u.attack;
      document.getElementById("inspect-arm").textContent = u.armor;
      document.getElementById("inspect-spd").textContent = u.speed;
      document.getElementById("inspect-pos").textContent = `(${u.x}, ${u.y})`;
      document.getElementById("inspect-role").textContent = u.role;
      document.getElementById("inspect-pref").textContent = u.pref;
      document.getElementById("inspect-intent").textContent = u.team === "allies" ? (u.guarded ? "Guarding (30% Def)" : "Standing Ready") : u.intent;
    }

    // 7. Narrative Log
    const logBox = document.getElementById("combat-log");
    logBox.innerHTML = "";
    this.session.log.forEach(item => {
      const entry = document.createElement("div");
      entry.className = `log-entry ${item.type}`;
      entry.innerHTML = `<span class="log-time">[${item.time}]</span> <span>${item.text}</span>`;
      logBox.appendChild(entry);
    });
    logBox.scrollTop = logBox.scrollHeight;
  }

  renderLoop() {
    this.render();
    requestAnimationFrame(() => this.renderLoop());
  }

  render() {
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    // Compute active ranges
    let reachableTiles = [];
    let validTargets = [];

    if (this.selectedUnit && this.selectedUnit.team === "allies" && this.selectedUnit.alive) {
      if (this.activeCommand === "move") {
        reachableTiles = this.session.getReachableTiles(this.selectedUnit);
      } else if (this.activeCommand === "attack" || this.activeCommand === "skill") {
        const range = this.activeCommand === "skill" ? this.selectedUnit.skill_range : 1;
        validTargets = this.session.getValidTargets(this.selectedUnit, range);
      }
    }

    // 1. Draw 12x9 Tactical Battlefield Grid
    for (let y = 0; y < this.session.GRID_H; y++) {
      for (let x = 0; x < this.session.GRID_W; x++) {
        const px = this.originX + x * this.tileW;
        const py = this.originY + y * this.tileH;
        const w = this.tileW - 3;
        const h = this.tileH - 3;

        const isHover = (this.hoverTile.x === x && this.hoverTile.y === y);
        const isReachableMove = reachableTiles.some(t => t.x === x && t.y === y);
        const isTargetTile = validTargets.some(t => t.x === x && t.y === y);
        const isPartyCol = (x === this.session.PARTY_COL);
        const isEnemyCol = (x === this.session.ENEMY_COL);

        // Tile Fill
        if (isTargetTile) {
          ctx.fillStyle = isHover ? "rgba(244, 63, 94, 0.45)" : "rgba(244, 63, 94, 0.25)";
        } else if (isReachableMove) {
          ctx.fillStyle = isHover ? "rgba(56, 189, 248, 0.45)" : "rgba(56, 189, 248, 0.20)";
        } else if (isHover) {
          ctx.fillStyle = "rgba(56, 189, 248, 0.12)";
        } else if ((x + y) % 2 === 0) {
          ctx.fillStyle = "#141924";
        } else {
          ctx.fillStyle = "#10141d";
        }
        ctx.fillRect(px, py, w, h);

        // Tile Borders
        if (isTargetTile) {
          ctx.strokeStyle = "#f43f5e";
          ctx.lineWidth = 1.5;
        } else if (isReachableMove) {
          ctx.strokeStyle = "#38bdf8";
          ctx.lineWidth = 1.5;
        } else if (isPartyCol) {
          ctx.strokeStyle = "rgba(56, 189, 248, 0.4)";
          ctx.lineWidth = 1.2;
        } else if (isEnemyCol) {
          ctx.strokeStyle = "rgba(244, 63, 94, 0.4)";
          ctx.lineWidth = 1.2;
        } else {
          ctx.strokeStyle = "#222a3a";
          ctx.lineWidth = 1;
        }
        ctx.strokeRect(px, py, w, h);

        // Coordinate text
        ctx.fillStyle = "#475569";
        ctx.font = "9px 'JetBrains Mono', monospace";
        ctx.fillText(`${x},${y}`, px + 4, py + 11);

        // Reachable footprint indicator
        if (isReachableMove) {
          ctx.fillStyle = "#38bdf8";
          ctx.font = "10px sans-serif";
          ctx.textAlign = "center";
          ctx.fillText("👣", px + w / 2, py + h / 2 + 4);
          ctx.textAlign = "left";
        }
      }
    }

    // 2. Draw Movement Path Preview Line
    if (this.activeCommand === "move" && this.selectedUnit && this.hoverTile.x >= 0) {
      const isReachable = reachableTiles.some(t => t.x === this.hoverTile.x && t.y === this.hoverTile.y);
      if (isReachable) {
        const startX = this.originX + this.selectedUnit.x * this.tileW + this.tileW / 2;
        const startY = this.originY + this.selectedUnit.y * this.tileH + this.tileH / 2;
        const targetX = this.originX + this.hoverTile.x * this.tileW + this.tileW / 2;
        const targetY = this.originY + this.hoverTile.y * this.tileH + this.tileH / 2;

        ctx.save();
        ctx.beginPath();
        ctx.setLineDash([5, 5]);
        ctx.moveTo(startX, startY);
        ctx.lineTo(targetX, targetY);
        ctx.strokeStyle = "#38bdf8";
        ctx.lineWidth = 2.5;
        ctx.stroke();
        ctx.restore();
      }
    }

    // 3. Draw Attack Targeting Line
    if ((this.activeCommand === "attack" || this.activeCommand === "skill") && this.selectedUnit && this.hoverTile.x >= 0) {
      const targetUnit = this.session.getUnitAt(this.hoverTile.x, this.hoverTile.y);
      if (targetUnit && validTargets.includes(targetUnit)) {
        const startX = this.originX + this.selectedUnit.x * this.tileW + this.tileW / 2;
        const startY = this.originY + this.selectedUnit.y * this.tileH + this.tileH / 2;
        const targetX = this.originX + targetUnit.x * this.tileW + this.tileW / 2;
        const targetY = this.originY + targetUnit.y * this.tileH + this.tileH / 2;

        ctx.save();
        ctx.beginPath();
        ctx.moveTo(startX, startY);
        ctx.lineTo(targetX, targetY);
        ctx.strokeStyle = "#f43f5e";
        ctx.lineWidth = 2.5;
        ctx.stroke();

        // Crosshair reticle over enemy
        ctx.beginPath();
        ctx.arc(targetX, targetY, 22, 0, Math.PI * 2);
        ctx.strokeStyle = "#fbbf24";
        ctx.lineWidth = 2;
        ctx.stroke();
        ctx.restore();
      }
    }

    // 4. Smooth Unit Position Interpolation & Token Rendering
    const drawUnit = (u, isAlly) => {
      if (!u.alive) return;

      // Smooth position interpolation (lerp)
      u.renderX += (u.x - u.renderX) * 0.22;
      u.renderY += (u.y - u.renderY) * 0.22;

      const cx = this.originX + u.renderX * this.tileW + (this.tileW - 3) / 2;
      const cy = this.originY + u.renderY * this.tileH + (this.tileH - 3) / 2;
      const isSelected = (this.selectedUnit === u);

      // Token Aura
      ctx.beginPath();
      ctx.arc(cx, cy, 18, 0, Math.PI * 2);
      ctx.fillStyle = isAlly ? "rgba(56, 189, 248, 0.25)" : "rgba(244, 63, 94, 0.25)";
      ctx.fill();

      // Guarded Shield Aura
      if (u.guarded) {
        ctx.beginPath();
        ctx.arc(cx, cy, 21, 0, Math.PI * 2);
        ctx.strokeStyle = "#38bdf8";
        ctx.lineWidth = 2;
        ctx.stroke();
      }

      // Outer Selection Ring
      if (isSelected) {
        ctx.beginPath();
        ctx.arc(cx, cy, 21, 0, Math.PI * 2);
        ctx.strokeStyle = "#fbbf24";
        ctx.lineWidth = 2.5;
        ctx.stroke();
      }

      // Token Body
      ctx.beginPath();
      ctx.arc(cx, cy, 15, 0, Math.PI * 2);
      ctx.fillStyle = isAlly ? "#0284c7" : "#be123c";
      ctx.fill();
      ctx.strokeStyle = isAlly ? "#38bdf8" : "#f43f5e";
      ctx.lineWidth = 2;
      ctx.stroke();

      // Badge letter
      ctx.fillStyle = "#ffffff";
      ctx.font = "bold 13px 'JetBrains Mono', monospace";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(u.badge, cx, cy);

      // Mini Health Bar above head
      const barW = this.tileW - 18;
      const barH = 4;
      const barX = cx - barW / 2;
      const barY = cy - 23;
      const ratio = Math.max(0, u.hp / u.max_hp);

      ctx.fillStyle = "rgba(0, 0, 0, 0.65)";
      ctx.fillRect(barX, barY, barW, barH);

      ctx.fillStyle = isAlly ? (ratio > 0.4 ? "#10b981" : "#ef4444") : "#f43f5e";
      ctx.fillRect(barX, barY, barW * ratio, barH);
    };

    this.session.allies.forEach(a => drawUnit(a, true));
    this.session.enemies.forEach(e => drawUnit(e, false));

    // 5. Floating Text FX
    for (let i = floatingTexts.length - 1; i >= 0; i--) {
      const ft = floatingTexts[i];
      const fx = this.originX + ft.gridX * this.tileW + this.tileW / 2;
      const fy = this.originY + ft.gridY * this.tileH + ft.yOffset;

      ctx.save();
      ctx.globalAlpha = ft.alpha;
      ctx.fillStyle = ft.color;
      ctx.font = "bold 15px 'Outfit', sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(ft.text, fx, fy);
      ctx.restore();

      ft.yOffset -= 0.9;
      ft.life -= 1;
      ft.alpha = Math.max(0, ft.life / 50);
      if (ft.life <= 0) floatingTexts.splice(i, 1);
    }

    // 6. Impact Particles FX
    for (let i = particles.length - 1; i >= 0; i--) {
      const p = particles[i];
      const px = this.originX + p.gridX * this.tileW + this.tileW / 2 + p.x;
      const py = this.originY + p.gridY * this.tileH + this.tileH / 2 + p.y;

      ctx.save();
      ctx.globalAlpha = p.alpha;
      ctx.fillStyle = p.color;
      ctx.beginPath();
      ctx.arc(px, py, 2.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();

      p.x += p.vx;
      p.y += p.vy;
      p.life -= 1;
      p.alpha = Math.max(0, p.life / 25);
      if (p.life <= 0) particles.splice(i, 1);
    }
  }
}

// Start application
window.addEventListener("DOMContentLoaded", () => {
  window.aegisApp = new AegisApp();
});
