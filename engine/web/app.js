/**
 * PROJECT AEGIS - 2D Tactical Web Runtime
 * Interactive Player GUI for Project Aegis Tactical RPG Engine.
 */

// --- 1. Audio Synthesizer (Web Audio API) ---
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

  playStrike() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sawtooth';
    osc.frequency.setValueAtTime(180, this.ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(40, this.ctx.currentTime + 0.12);
    gain.gain.setValueAtTime(0.2, this.ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, this.ctx.currentTime + 0.12);
    osc.connect(gain);
    gain.connect(this.ctx.destination);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.12);
  }

  playBow() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(520, this.ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(220, this.ctx.currentTime + 0.18);
    gain.gain.setValueAtTime(0.18, this.ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, this.ctx.currentTime + 0.18);
    osc.connect(gain);
    gain.connect(this.ctx.destination);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.18);
  }

  playDefeat() {
    if (this.muted) return;
    this.init();
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'triangle';
    osc.frequency.setValueAtTime(120, this.ctx.currentTime);
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
      gain.gain.setValueAtTime(0.15, this.ctx.currentTime + i * 0.1);
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
    warrior: { name: "Warrior", class_id: "Warrior", max_hp: 242, attack: 41, armor: 18, speed: 10, badge: "W", role: "Vanguard", pref: "Nearest Foe" },
    dojo_warrior: { name: "Veteran Warrior", class_id: "Warrior", max_hp: 242, attack: 41, armor: 18, speed: 10, badge: "W", role: "Vanguard", pref: "Nearest Foe" },
    ranger: { name: "Ranger", class_id: "Ranger", max_hp: 158, attack: 35, armor: 6, speed: 12, badge: "R", role: "Marksman", pref: "Lowest HP Foe" },
    dojo_ranger: { name: "Veteran Ranger", class_id: "Ranger", max_hp: 158, attack: 35, armor: 6, speed: 12, badge: "R", role: "Marksman", pref: "Lowest HP Foe" },
    priest: { name: "Priest", class_id: "Priest", max_hp: 176, attack: 34, armor: 5, speed: 9, badge: "P", role: "Support", pref: "Allied Buffer" },
    
    goblin_brawler: { name: "Goblin Brawler", class_id: "Brawler", max_hp: 108, attack: 31, armor: 9, speed: 8, badge: "B", role: "Bruiser", pref: "Frontline" },
    enemy_brawler: { name: "Goblin Brawler", class_id: "Brawler", max_hp: 108, attack: 31, armor: 9, speed: 8, badge: "B", role: "Bruiser", pref: "Frontline" },
    goblin_skirmisher: { name: "Goblin Skirmisher", class_id: "Rogue", max_hp: 65, attack: 32, armor: 7, speed: 14, badge: "S", role: "Flanker", pref: "Squishy Target" },
    fleeing_scout: { name: "Goblin Scout", class_id: "Rogue", max_hp: 65, attack: 32, armor: 7, speed: 14, badge: "S", role: "Scout", pref: "Escape / Flank" },
    goblin_archer: { name: "Goblin Archer", class_id: "Ranger", max_hp: 71, attack: 35, armor: 7, speed: 13, badge: "A", role: "Ranged", pref: "Warrior" },
    goblin_archer_2: { name: "Goblin Archer", class_id: "Ranger", max_hp: 71, attack: 35, armor: 7, speed: 13, badge: "A", role: "Ranged", pref: "Ranger" },
    goblin_shaman: { name: "Goblin Shaman", class_id: "Mage", max_hp: 77, attack: 25, armor: 6, speed: 7, badge: "M", role: "Caster", pref: "Weakest Hero" },
    goblin_berserker: { name: "Goblin Berserker", class_id: "Berserker", max_hp: 108, attack: 31, armor: 5, speed: 11, badge: "Z", role: "Berserker", pref: "Highest Threat" }
  }
};

// --- 3. Simulation & Combat Session ---
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
    this.outcome = "ongoing"; // "ongoing" | "victory" | "defeat"

    this.allies = [];
    this.enemies = [];
    this.log = [];

    this.initCombatants();
  }

  initCombatants() {
    const allyRows = [4, 3, 5, 2, 6];
    this.allies = this.encounter.party.map((id, index) => {
      const proto = CATALOG.units[id] || CATALOG.units.warrior;
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
        x: this.PARTY_COL,
        y: allyRows[index % allyRows.length],
        alive: true,
        renderX: this.PARTY_COL,
        renderY: allyRows[index % allyRows.length],
        animOffset: { x: 0, y: 0 }
      };
    });

    const enemyRows = [4, 3, 5, 2, 6];
    this.enemies = this.encounter.enemies.map((id, index) => {
      const proto = CATALOG.units[id] || CATALOG.units.goblin_brawler;
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
        x: this.ENEMY_COL,
        y: enemyRows[index % enemyRows.length],
        alive: true,
        renderX: this.ENEMY_COL,
        renderY: enemyRows[index % enemyRows.length],
        animOffset: { x: 0, y: 0 }
      };
    });
  }

  resolveRound(audio) {
    if (this.outcome !== "ongoing" || this.round > this.max_rounds) return;

    this.logMessage(`--- Round ${this.round} begins ---`, "round-banner");

    // Allies turn
    for (const ally of this.allies) {
      if (!ally.alive) continue;
      const target = this.findLiveTarget(this.enemies);
      if (!target) break;

      const dmg = Math.max(1, ally.attack - target.armor);
      target.hp = Math.max(0, target.hp - dmg);
      
      // Visual lunge & sound
      ally.animOffset = { x: 14, y: 0 };
      if (ally.badge === "R") audio.playBow(); else audio.playStrike();

      spawnFloatingText(target.x, target.y, `-${dmg}`, "#fbbf24");
      spawnParticles(target.x, target.y, "#fbbf24");

      this.logMessage(`[Block 1] ${ally.name} strikes ${target.name} for ${dmg} dmg (${target.hp} HP remaining)`, "ally-action");

      if (target.hp === 0) {
        target.alive = false;
        audio.playDefeat();
        this.logMessage(`*** [DEFEAT] ${target.name} has fallen! ***`, "defeat-banner");
      }
    }

    // Enemies turn
    for (const enemy of this.enemies) {
      if (!enemy.alive) continue;
      const target = this.findLiveTarget(this.allies);
      if (!target) break;

      const dmg = Math.max(1, enemy.attack - target.armor);
      target.hp = Math.max(0, target.hp - dmg);

      enemy.animOffset = { x: -14, y: 0 };
      audio.playStrike();

      spawnFloatingText(target.x, target.y, `-${dmg}`, "#f43f5e");
      spawnParticles(target.x, target.y, "#f43f5e");

      this.logMessage(`[Block 1] ${enemy.name} strikes ${target.name} for ${dmg} dmg (${target.hp} HP remaining)`, "enemy-action");

      if (target.hp === 0) {
        target.alive = false;
        audio.playDefeat();
        this.logMessage(`*** [DEFEAT] ${target.name} has fallen! ***`, "defeat-banner");
      }
    }

    // Check Outcome
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
      this.logMessage("=== DRAW: Round limit reached. ===", "round-banner");
    } else {
      this.round += 1;
    }
  }

  findLiveTarget(list) {
    return list.find(u => u.alive) || null;
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
    life: 45
  });
}

function spawnParticles(gridX, gridY, color) {
  for (let i = 0; i < 8; i++) {
    const angle = (Math.PI * 2 * i) / 8 + Math.random() * 0.5;
    const speed = 1.5 + Math.random() * 2.5;
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
    this.hoverTile = { x: -1, y: -1 };
    this.autoplayInterval = null;

    this.tileW = 66;
    this.tileH = 58;
    this.originX = 24;
    this.originY = 50;

    this.initEventListeners();
    this.updateUI();
    this.renderLoop();
  }

  initEventListeners() {
    // Buttons
    document.getElementById("btn-advance").addEventListener("click", () => {
      this.session.resolveRound(this.audio);
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
        }, 650);
      }
    });

    document.getElementById("btn-reset").addEventListener("click", () => {
      const key = document.getElementById("encounter-select").value;
      this.session = new CombatSession(key);
      this.selectedUnit = this.session.allies[0];
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
        allies: this.session.allies.map(a => ({ id: a.id, hp: a.hp, alive: a.alive }))
      };
      localStorage.setItem("aegis_slot_0", JSON.stringify(data));
      this.session.logMessage("[Engine] Saved battle state to slot_0 (Local Storage)", "round-banner");
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
      this.updateUI();
    });

    document.getElementById("btn-clear-log").addEventListener("click", () => {
      this.session.log = [];
      this.updateUI();
    });

    // Canvas Mouse Interaction
    this.canvas.addEventListener("mousemove", (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const gx = Math.floor((mouseX - this.originX) / this.tileW);
      const gy = Math.floor((mouseY - this.originY) / this.tileH);

      if (gx >= 0 && gx < this.session.GRID_W && gy >= 0 && gy < this.session.GRID_H) {
        this.hoverTile = { x: gx, y: gy };
        document.getElementById("cursor-coords").textContent = `Tile: [${gx}, ${gy}]`;
      } else {
        this.hoverTile = { x: -1, y: -1 };
        document.getElementById("cursor-coords").textContent = `Tile: [—, —]`;
      }
    });

    this.canvas.addEventListener("click", () => {
      const { x, y } = this.hoverTile;
      if (x < 0 || y < 0) return;

      const clickedAlly = this.session.allies.find(a => a.x === x && a.y === y && a.alive);
      const clickedEnemy = this.enemiesAt(x, y);

      if (clickedAlly) {
        this.selectedUnit = clickedAlly;
        this.updateUI();
      } else if (clickedEnemy) {
        this.selectedUnit = clickedEnemy;
        this.updateUI();
      }
    });
  }

  enemiesAt(x, y) {
    return this.session.enemies.find(e => e.x === x && e.y === y && e.alive);
  }

  updateUI() {
    // 1. Round Display & Verdict
    const roundDisp = document.getElementById("round-display");
    roundDisp.textContent = `${this.session.round} / ${this.session.max_rounds}`;

    const verdictDisp = document.getElementById("verdict-display");
    verdictDisp.className = `pill-status ${this.session.outcome}`;
    verdictDisp.textContent = this.session.outcome.toUpperCase();

    // 2. Arena Title
    document.getElementById("arena-name").textContent =
      `World: Ashlands • Zone: ${this.session.encounter.zone}`;

    // 3. Allies Roster List
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
              <div class="unit-class-badge">${ally.class_id}</div>
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
        </div>
      `;

      card.addEventListener("click", () => {
        this.selectedUnit = ally;
        this.updateUI();
      });

      alliesContainer.appendChild(card);
    });

    document.getElementById("party-count").textContent = `${liveAllies} ALIVE`;

    // 4. Enemies Roster List
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
              <div class="unit-class-badge">${enemy.class_id}</div>
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
        this.updateUI();
      });

      enemiesContainer.appendChild(card);
    });

    document.getElementById("enemy-count").textContent = `${liveEnemies} ALIVE`;

    // 5. Unit Inspector Card
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
    }

    // 6. Narrative Log Feed
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

    // 1. Draw 12x9 Tactical Tiles
    for (let y = 0; y < this.session.GRID_H; y++) {
      for (let x = 0; x < this.session.GRID_W; x++) {
        const px = this.originX + x * this.tileW;
        const py = this.originY + y * this.tileH;
        const w = this.tileW - 4;
        const h = this.tileH - 4;

        const isHover = (this.hoverTile.x === x && this.hoverTile.y === y);
        const isPartyCol = (x === this.session.PARTY_COL);
        const isEnemyCol = (x === this.session.ENEMY_COL);

        // Tile background
        if (isHover) {
          ctx.fillStyle = "rgba(56, 189, 248, 0.15)";
        } else if ((x + y) % 2 === 0) {
          ctx.fillStyle = "#151b27";
        } else {
          ctx.fillStyle = "#111620";
        }
        ctx.fillRect(px, py, w, h);

        // Tile borders
        if (isPartyCol) {
          ctx.strokeStyle = "rgba(56, 189, 248, 0.4)";
          ctx.lineWidth = 1.5;
        } else if (isEnemyCol) {
          ctx.strokeStyle = "rgba(244, 63, 94, 0.4)";
          ctx.lineWidth = 1.5;
        } else {
          ctx.strokeStyle = "#242c3d";
          ctx.lineWidth = 1;
        }
        ctx.strokeRect(px, py, w, h);

        // Coordinate stamp
        ctx.fillStyle = "#475569";
        ctx.font = "9px 'JetBrains Mono', monospace";
        ctx.fillText(`${x},${y}`, px + 4, py + 12);
      }
    }

    // 2. Draw Units
    const drawUnitToken = (unit, isAlly) => {
      if (!unit.alive) return;

      // Smooth animation decay
      unit.animOffset.x *= 0.82;
      unit.animOffset.y *= 0.82;

      const cx = this.originX + unit.x * this.tileW + (this.tileW - 4) / 2 + unit.animOffset.x;
      const cy = this.originY + unit.y * this.tileH + (this.tileH - 4) / 2 + unit.animOffset.y;
      const isSelected = (this.selectedUnit === unit);

      // Token Aura
      ctx.beginPath();
      ctx.arc(cx, cy, 18, 0, Math.PI * 2);
      ctx.fillStyle = isAlly ? "rgba(14, 165, 233, 0.25)" : "rgba(244, 63, 94, 0.25)";
      ctx.fill();

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
      ctx.arc(cx, cy, 16, 0, Math.PI * 2);
      ctx.fillStyle = isAlly ? "#0284c7" : "#be123c";
      ctx.fill();
      ctx.strokeStyle = isAlly ? "#38bdf8" : "#f43f5e";
      ctx.lineWidth = 2;
      ctx.stroke();

      // Unit Badge Letter
      ctx.fillStyle = "#ffffff";
      ctx.font = "bold 13px 'JetBrains Mono', monospace";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(unit.badge, cx, cy);

      // Mini Health Bar above head
      const barW = this.tileW - 16;
      const barH = 4;
      const barX = cx - barW / 2;
      const barY = cy - 24;
      const ratio = Math.max(0, unit.hp / unit.max_hp);

      ctx.fillStyle = "rgba(0, 0, 0, 0.6)";
      ctx.fillRect(barX, barY, barW, barH);

      ctx.fillStyle = isAlly ? (ratio > 0.4 ? "#10b981" : "#ef4444") : "#f43f5e";
      ctx.fillRect(barX, barY, barW * ratio, barH);
    };

    this.session.allies.forEach(a => drawUnitToken(a, true));
    this.session.enemies.forEach(e => drawUnitToken(e, false));

    // 3. Render Floating Texts
    for (let i = floatingTexts.length - 1; i >= 0; i--) {
      const ft = floatingTexts[i];
      const fx = this.originX + ft.gridX * this.tileW + this.tileW / 2;
      const fy = this.originY + ft.gridY * this.tileH + ft.yOffset;

      ctx.save();
      ctx.globalAlpha = ft.alpha;
      ctx.fillStyle = ft.color;
      ctx.font = "bold 16px 'Outfit', sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(ft.text, fx, fy);
      ctx.restore();

      ft.yOffset -= 0.8;
      ft.life -= 1;
      ft.alpha = Math.max(0, ft.life / 45);
      if (ft.life <= 0) floatingTexts.splice(i, 1);
    }

    // 4. Render Particles
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

// Start application on DOM load
window.addEventListener("DOMContentLoaded", () => {
  window.aegisApp = new AegisApp();
});
