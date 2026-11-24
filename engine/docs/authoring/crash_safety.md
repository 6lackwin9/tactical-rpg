# Crash safety: the report, the rescue save and damaged saves

Status: canon (ENGINE-62, CRASH-SAFE1). Game-agnostic. Every game on the shell gets the report, the line at the next
start and the save fallbacks with no code of its own. Only the rescue save needs one method from the game.

## What happens on a panic

1. The panic hook (`aegis_ui::crashlog::arm`, set when the shell opens) notes the panic. With a log open (a windowed
   run), it writes `crash-<unix seconds>.txt` in `<user dir>/logs/`. The report holds the message and place, the
   build (dev or release, the engine version and `AEGIS_COMMIT` when the build sets it), the top screen, the last 200
   log lines, the backtrace and the last lines of `feel.log` when one is there.
2. The shell catches the panic as it comes back from the event or frame: a window's event handler, or a no-window
   run's request or frame. The Windows gamepad poll runs inside the same catch (ENGINE-63). It calls `Context::rescue`, which writes the rescue save. Then `crashlog::crashed` writes
   the report if none is written yet (a no-window run has no log) and leaves `crashed.json` beside it. After that the
   panic goes on as before, and the program ends.
3. At the next start the shell reads and removes `crashed.json`. It shows one line over the first screen: "The game
   closed unexpectedly. A report is at `PATH`." The line has **Open folder** and **OK** buttons (screen id
   `crash_note`).

## The rescue save (the one game hook)

```rust
fn rescue(&self) -> Option<Vec<String>> {
    let game = self.game.as_ref()?; // nothing in play: no rescue
    Some(my_save::write_keyed(&self.data, &self.save_dir, &game.state, aegis_save::RESCUE_KEY, 0))
}
```

- Write the game in play and nothing more. It runs after a panic, so the state may be half changed. Keep it to one
  save write.
- It goes to the keyed save `aegis_save::RESCUE_KEY` (`key_rescue.json`). Keyed saves sit outside the slot range, so a
  rescue never lands on a player's slot. A panic inside `rescue` itself is caught and logged.
- Offer it at the title next to Continue, and remove it once taken (`aegis_save::delete_keyed`). A game without the
  method gets the report and the line, but no rescue.

## Damaged saves and the previous copy

- The atomic write (`aegis_save::write`, `write_keyed`) keeps the save it replaces as `<file>.prev`
  (`slot_1.json.prev`), but only when that save reads. A damaged save is never copied over a good copy.
- `aegis_save::read` returns the previous copy's text when the slot's file is cut short or garbled. `damage(fmt, dir,
  slot)` says why the file is damaged. `peek_slot` gives the header from the copy, along with the damage reason.
- A damaged save is never deleted by a read. Deleting the slot on purpose (`delete`) removes its copy too.
- The RPG kit's `save::read` also loads the copy when the slot reads but fails later (a refused version, a failed
  migration), with a warning. Its `peek` adds `damaged`, so a slot card can show "Damaged (reason)".

## Testing it

- A dev build's agent link takes `{"cmd": "panic", "message": "..."}`. It forces a panic in a no-window run, which
  exercises the whole path. A release build refuses the command.
- See `engine/crates/aegis_ui/tests/crash_safe.rs` (report, rescue and the next start),
  `engine/crates/aegis_save/tests/prev_copy.rs` (the copy and damage), and the sample game's
  `games/hello/crates/hello_game/tests/crash.rs`.
