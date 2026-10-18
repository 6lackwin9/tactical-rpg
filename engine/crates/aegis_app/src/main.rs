mod layout;
mod renderer;

use aegis_core::IVec2;
use aegis_vfs::{CatalogStore, Vfs};
use aegis_vm::{BattleOutcome, CombatGrid, CombatSession, Combatant, Team};
use layout::AppLayout;
use renderer::WgpuRenderer;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<WgpuRenderer>,
    layout: AppLayout,
    session: CombatSession,
    catalog: CatalogStore,
    vfs: Vfs,
    selected_unit_id: Option<String>,
    cursor_pos: [f32; 2],
    hover_tile: Option<IVec2>,
    autoplay: bool,
    last_step: Instant,
    outcome: BattleOutcome,
}

impl App {
    fn new() -> Self {
        let vfs = Vfs::discover();
        let mut catalog = CatalogStore::new(vfs.clone());
        let _ = catalog.load_all_standard();

        let party = vec![
            Combatant::new("warrior", "Alda (Warrior)", Team::Allies, 242, 41, 18, 10),
            Combatant::new("ranger", "Bryn (Ranger)", Team::Allies, 158, 35, 6, 12),
            Combatant::new("priest", "Dagny (Priest)", Team::Allies, 176, 34, 5, 9),
        ];

        let enemies = vec![
            Combatant::new("goblin_brawler", "Goblin Brawler", Team::Enemies, 108, 31, 9, 8),
            Combatant::new("goblin_skirmisher", "Goblin Skirmisher", Team::Enemies, 65, 32, 7, 14),
            Combatant::new("goblin_archer", "Goblin Archer", Team::Enemies, 71, 35, 7, 13),
            Combatant::new("goblin_shaman", "Goblin Shaman", Team::Enemies, 77, 25, 6, 7),
            Combatant::new("goblin_berserker", "Goblin Berserker", Team::Enemies, 108, 31, 5, 11),
        ];

        let session = CombatSession::new(party, enemies, 1337);

        Self {
            window: None,
            renderer: None,
            layout: AppLayout::new(),
            session,
            catalog,
            vfs,
            selected_unit_id: Some("warrior".to_string()),
            cursor_pos: [0.0, 0.0],
            hover_tile: None,
            autoplay: false,
            last_step: Instant::now(),
            outcome: BattleOutcome::Ongoing,
        }
    }

    fn advance_round(&mut self) {
        if self.outcome == BattleOutcome::Ongoing && self.session.current_round <= self.session.max_rounds {
            println!("[Aegis WGPU] Advancing Round {}/{}...", self.session.current_round, self.session.max_rounds);
            self.outcome = self.session.resolve_round_autoplay();
        }
    }

    fn reset_battle(&mut self) {
        let party = vec![
            Combatant::new("warrior", "Alda (Warrior)", Team::Allies, 242, 41, 18, 10),
            Combatant::new("ranger", "Bryn (Ranger)", Team::Allies, 158, 35, 6, 12),
            Combatant::new("priest", "Dagny (Priest)", Team::Allies, 176, 34, 5, 9),
        ];

        let enemies = vec![
            Combatant::new("goblin_brawler", "Goblin Brawler", Team::Enemies, 108, 31, 9, 8),
            Combatant::new("goblin_skirmisher", "Goblin Skirmisher", Team::Enemies, 65, 32, 7, 14),
            Combatant::new("goblin_archer", "Goblin Archer", Team::Enemies, 71, 35, 7, 13),
            Combatant::new("goblin_shaman", "Goblin Shaman", Team::Enemies, 77, 25, 6, 7),
            Combatant::new("goblin_berserker", "Goblin Berserker", Team::Enemies, 108, 31, 5, 11),
        ];

        self.session = CombatSession::new(party, enemies, 1337);
        self.outcome = BattleOutcome::Ongoing;
        self.autoplay = false;
        println!("[Aegis WGPU] Reset combat session to Round 1.");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_title("Project Aegis - 2D Tactical Engine (WGPU / Taffy)")
                .with_inner_size(PhysicalSize::new(1280, 760));

            let window = Arc::new(event_loop.create_window(win_attr).unwrap());
            self.window = Some(window.clone());

            let renderer = pollster::block_on(WgpuRenderer::new(window)).unwrap();
            self.renderer = Some(renderer);
            println!("==================================================================");
            println!("===      PROJECT AEGIS - WGPU NATIVE DESKTOP SHELL (v0.1)      ===");
            println!("===     Running Hardware-Accelerated 2D Tactical Arena        ===");
            println!("==================================================================");
            println!("Controls: [Space] Advance Round | [A] Autoplay | [R] Reset | [Esc] Exit");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = [position.x as f32, position.y as f32];
            }

            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                if let Some(tile) = self.hover_tile {
                    // Check if clicked an ally
                    if let Some(ally) = self.session.allies.iter().find(|a| a.position == tile && a.is_alive()) {
                        self.selected_unit_id = Some(ally.id.clone());
                        println!("[Inspect] Selected Ally: {} (HP: {}/{})", ally.name, ally.current_hp, ally.max_hp);
                    } else if let Some(enemy) = self.session.enemies.iter().find(|e| e.position == tile && e.is_alive()) {
                        self.selected_unit_id = Some(enemy.id.clone());
                        println!("[Inspect] Selected Enemy: {} (HP: {}/{})", enemy.name, enemy.current_hp, enemy.max_hp);
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => match key {
                KeyCode::Space | KeyCode::Enter => {
                    self.advance_round();
                }
                KeyCode::KeyA => {
                    self.autoplay = !self.autoplay;
                    println!("[Aegis WGPU] Autoplay toggled: {}", self.autoplay);
                }
                KeyCode::KeyR => {
                    self.reset_battle();
                }
                KeyCode::Escape => {
                    event_loop.exit();
                }
                _ => {}
            },

            WindowEvent::RedrawRequested => {
                // Autoplay timer
                if self.autoplay && self.outcome == BattleOutcome::Ongoing {
                    if self.last_step.elapsed() >= Duration::from_millis(600) {
                        self.advance_round();
                        self.last_step = Instant::now();
                    }
                }

                // Render frame
                if let Some(renderer) = &mut self.renderer {
                    let w = renderer.size.width as f32;
                    let h = renderer.size.height as f32;

                    self.layout.compute(w, h);
                    renderer.clear_batch();

                    // 1. Draw Top Control Bar
                    let top_layout = self.layout.taffy.layout(self.layout.top_bar).unwrap();
                    renderer.add_rect(
                        top_layout.location.x,
                        top_layout.location.y,
                        top_layout.size.width,
                        top_layout.size.height,
                        [0.08, 0.10, 0.15, 0.95],
                    );
                    // Top bar border line
                    renderer.add_rect(
                        top_layout.location.x,
                        top_layout.location.y + top_layout.size.height - 2.0,
                        top_layout.size.width,
                        2.0,
                        [0.22, 0.45, 0.95, 0.8],
                    );

                    // 2. Draw Left Panel (Allies)
                    let left_layout = self.layout.taffy.layout(self.layout.left_panel).unwrap();
                    renderer.add_rect(
                        left_layout.location.x,
                        left_layout.location.y,
                        left_layout.size.width,
                        left_layout.size.height,
                        [0.06, 0.08, 0.12, 0.9],
                    );

                    // Allied cards
                    for (i, ally) in self.session.allies.iter().enumerate() {
                        let card_y = left_layout.location.y + 12.0 + (i as f32 * 72.0);
                        let is_selected = self.selected_unit_id.as_deref() == Some(&ally.id);
                        let card_color = if is_selected {
                            [0.12, 0.18, 0.28, 0.95]
                        } else {
                            [0.09, 0.12, 0.18, 0.85]
                        };
                        renderer.add_rect(left_layout.location.x + 8.0, card_y, 244.0, 64.0, card_color);

                        // Card accent indicator
                        renderer.add_rect(left_layout.location.x + 8.0, card_y, 4.0, 64.0, [0.22, 0.65, 0.95, 1.0]);

                        // Ally HP bar
                        let ratio = (ally.current_hp as f32 / ally.max_hp as f32).clamp(0.0, 1.0);
                        renderer.add_rect(left_layout.location.x + 20.0, card_y + 46.0, 220.0, 8.0, [0.15, 0.18, 0.22, 1.0]);
                        renderer.add_rect(left_layout.location.x + 20.0, card_y + 46.0, 220.0 * ratio, 8.0, [0.15, 0.85, 0.40, 1.0]);
                    }

                    // 3. Draw Right Panel (Enemies)
                    let right_layout = self.layout.taffy.layout(self.layout.right_panel).unwrap();
                    renderer.add_rect(
                        right_layout.location.x,
                        right_layout.location.y,
                        right_layout.size.width,
                        right_layout.size.height,
                        [0.06, 0.08, 0.12, 0.9],
                    );

                    for (i, enemy) in self.session.enemies.iter().enumerate() {
                        let card_y = right_layout.location.y + 12.0 + (i as f32 * 68.0);
                        let is_selected = self.selected_unit_id.as_deref() == Some(&enemy.id);
                        let card_color = if is_selected {
                            [0.24, 0.12, 0.14, 0.95]
                        } else {
                            [0.09, 0.12, 0.18, 0.85]
                        };
                        renderer.add_rect(right_layout.location.x + 8.0, card_y, 244.0, 60.0, card_color);
                        renderer.add_rect(right_layout.location.x + 8.0, card_y, 4.0, 60.0, [0.95, 0.25, 0.35, 1.0]);

                        let ratio = (enemy.current_hp as f32 / enemy.max_hp as f32).clamp(0.0, 1.0);
                        renderer.add_rect(right_layout.location.x + 20.0, card_y + 44.0, 220.0, 6.0, [0.15, 0.18, 0.22, 1.0]);
                        renderer.add_rect(right_layout.location.x + 20.0, card_y + 44.0, 220.0 * ratio, 6.0, [0.95, 0.25, 0.35, 1.0]);
                    }

                    // 4. Draw Center 12x9 Tactical Grid
                    let center_layout = self.layout.taffy.layout(self.layout.center_arena).unwrap();
                    let arena_w = center_layout.size.width;
                    let arena_h = center_layout.size.height;
                    let tile_w = (arena_w - 40.0) / 12.0;
                    let tile_h = (arena_h - 40.0) / 9.0;
                    let origin_x = center_layout.location.x + 20.0;
                    let origin_y = center_layout.location.y + 20.0;

                    // Compute hover tile
                    let mx = self.cursor_pos[0] - origin_x;
                    let my = self.cursor_pos[1] - origin_y;
                    let gx = (mx / tile_w).floor() as i32;
                    let gy = (my / tile_h).floor() as i32;

                    if gx >= 0 && gx < 12 && gy >= 0 && gy < 9 {
                        self.hover_tile = Some(IVec2::new(gx, gy));
                    } else {
                        self.hover_tile = None;
                    }

                    // Draw grid tiles
                    for y in 0..CombatGrid::GRID_H {
                        for x in 0..CombatGrid::GRID_W {
                            let tx = origin_x + (x as f32 * tile_w);
                            let ty = origin_y + (y as f32 * tile_h);
                            let tw = tile_w - 3.0;
                            let th = tile_h - 3.0;

                            let is_hover = self.hover_tile == Some(IVec2::new(x, y));
                            let is_party_front = x == CombatGrid::PARTY_FRONT_X;
                            let is_enemy_front = x == CombatGrid::ENEMY_FRONT_X;

                            let base_color = if is_hover {
                                [0.20, 0.30, 0.45, 0.8]
                            } else if (x + y) % 2 == 0 {
                                [0.08, 0.10, 0.14, 0.95]
                            } else {
                                [0.06, 0.08, 0.11, 0.95]
                            };
                            renderer.add_rect(tx, ty, tw, th, base_color);

                            // Front line indicators
                            if is_party_front {
                                renderer.add_rect(tx + tw - 2.0, ty, 2.0, th, [0.22, 0.55, 0.95, 0.6]);
                            } else if is_enemy_front {
                                renderer.add_rect(tx, ty, 2.0, th, [0.95, 0.25, 0.35, 0.6]);
                            }

                            let coord = IVec2::new(x, y);

                            // Draw Ally Token
                            if let Some(ally) = self.session.allies.iter().find(|a| a.position == coord && a.is_alive()) {
                                let is_selected = self.selected_unit_id.as_deref() == Some(&ally.id);
                                let unit_x = tx + (tw - 32.0) / 2.0;
                                let unit_y = ty + (th - 32.0) / 2.0;

                                if is_selected {
                                    renderer.add_rect(unit_x - 3.0, unit_y - 3.0, 38.0, 38.0, [0.98, 0.85, 0.25, 0.9]);
                                }

                                renderer.add_rect(unit_x, unit_y, 32.0, 32.0, [0.15, 0.45, 0.90, 1.0]);

                                // Overhead HP mini-bar
                                let ratio = (ally.current_hp as f32 / ally.max_hp as f32).clamp(0.0, 1.0);
                                renderer.add_rect(tx + 4.0, ty + 4.0, tw - 8.0, 3.0, [0.15, 0.18, 0.22, 1.0]);
                                renderer.add_rect(tx + 4.0, ty + 4.0, (tw - 8.0) * ratio, 3.0, [0.15, 0.85, 0.40, 1.0]);
                            }

                            // Draw Enemy Token
                            if let Some(enemy) = self.session.enemies.iter().find(|e| e.position == coord && e.is_alive()) {
                                let is_selected = self.selected_unit_id.as_deref() == Some(&enemy.id);
                                let unit_x = tx + (tw - 32.0) / 2.0;
                                let unit_y = ty + (th - 32.0) / 2.0;

                                if is_selected {
                                    renderer.add_rect(unit_x - 3.0, unit_y - 3.0, 38.0, 38.0, [0.98, 0.85, 0.25, 0.9]);
                                }

                                renderer.add_rect(unit_x, unit_y, 32.0, 32.0, [0.85, 0.20, 0.25, 1.0]);

                                let ratio = (enemy.current_hp as f32 / enemy.max_hp as f32).clamp(0.0, 1.0);
                                renderer.add_rect(tx + 4.0, ty + 4.0, tw - 8.0, 3.0, [0.15, 0.18, 0.22, 1.0]);
                                renderer.add_rect(tx + 4.0, ty + 4.0, (tw - 8.0) * ratio, 3.0, [0.95, 0.25, 0.35, 1.0]);
                            }
                        }
                    }

                    // 5. Draw Bottom Narrative Log Panel
                    let bottom_layout = self.layout.taffy.layout(self.layout.bottom_panel).unwrap();
                    renderer.add_rect(
                        bottom_layout.location.x,
                        bottom_layout.location.y,
                        bottom_layout.size.width,
                        bottom_layout.size.height,
                        [0.05, 0.07, 0.10, 0.95],
                    );
                    renderer.add_rect(
                        bottom_layout.location.x,
                        bottom_layout.location.y,
                        bottom_layout.size.width,
                        1.0,
                        [0.15, 0.20, 0.28, 0.8],
                    );

                    let _ = renderer.render();
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
