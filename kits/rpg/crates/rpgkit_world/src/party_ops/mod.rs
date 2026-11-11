//! Party edits (PR5-31 `LAYER-UP-PARTY`; engine-only, no Godot original beyond the models each file names): the pure
//! rules a party screen edits a member's draft by, moved out of the game's party screen with no rule change so every
//! screen (party, Dojo) and test applies one rule. `deck` holds the deck rule's editing terms (`DeckTerms`: bucket caps,
//! auto-pinning, `channeled` pruning, orders and follow-up lists); `slots` the classic class + extra skill slots and
//! their ranks (`SkillSlots`, `can_assign_to_slot`); `gear` the equipment and pouch moves on a gear draft (two-handed
//! off-hand stow and clear, pouch stack targets). The squad's active size lives with the roster (`roster::toggle_active`).
//! Every edit returns the new draft or the refusal text (`Err`), as the screens show it.

pub mod deck;
pub mod gear;
pub mod slots;

pub use deck::{intent_names, intent_title, DeckTerms, DECK_KINDS};
pub use gear::{drop_move, equip, main_weapon, move_slot, named, pack, pouch_put, pouch_take, stack_put, take_off, unequip, Stash};
pub use slots::{can_assign_to_slot, draft_rank, path_families, rank_step, slot_cap, top_rank, SkillSlots};

/// A slot or intent id as a title (`main_hand` -> `Main Hand`): the words of the refusal texts.
pub fn title_case(slot: &str) -> String {
    slot.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
