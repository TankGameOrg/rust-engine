use std::collections::HashMap;
use crate::rule::action::{ActionDescription, ActionProvider};
use crate::rule::context::Context;
use crate::ruleset::response::Response;
use crate::ruleset::ruleset::{Ruleset, RulesetProvider};
use crate::state::board::Tile;
use crate::state::element::new_wall;
use crate::state::position::Position;
use crate::state::state::State;
use crate::util::attributes::{ACTIONS, DAMAGE, DURABILITY, GOLD, WALKABLE};

struct DefaultV3Actions;

impl DefaultV3Actions {
    pub const fn new() -> DefaultV3Actions {
        DefaultV3Actions
    }
}

impl ActionProvider for DefaultV3Actions {
    fn actions(&self) -> HashMap<String, ActionDescription> {
        HashMap::new()
    }
}

pub struct DefaultV3;

impl DefaultV3 {
    pub fn ruleset() -> Ruleset {
        static RULESET: DefaultV3 = DefaultV3;
        Ruleset::new(Box::new(&RULESET))
    }

    pub fn handle_unit_destroy(&self, context: &mut Context, position: &Position) {
        let class = {
            context
                .state
                .board()
                .get_unit_unsafe(position)
                .expect("Position out of bounds")
                .container()
                .get_class()
        };

        match class {
            Some(class) => match class.as_str() {
                "Tank" => {
                    let loot = context
                        .state
                        .board()
                        .get_unit_unsafe(position)
                        .expect("Position out of bounds")
                        .container()
                        .get_or_else(&GOLD, 0i32);

                    let subject = context
                        .state
                        .board_mut()
                        .get_tank_for_ref_mut(&context.player);
                    match subject {
                        Some(subject) => {
                            let subject_container = subject.mut_container();
                            let previous_gold = subject_container.get_or_else(&GOLD, 0i32);
                            subject_container.put(&GOLD, previous_gold + loot);
                        }
                        None => {}
                    }
                    context
                        .state
                        .board_mut()
                        .put_unit(position, Some(new_wall(2)));
                }
                _ => context.state.board_mut().put_unit(position, None),
            },
            None => context.state.board_mut().put_unit(position, None),
        }
    }

    pub fn handle_floor_destroy(&self, context: &mut Context, position: &Position) {
        let destroyed_container = context
            .state
            .board_mut()
            .get_floor_mut_unsafe(position)
            .expect("Position out of bounds")
            .mut_container();
        match destroyed_container.get_class() {
            Some(class) => match class.as_str() {
                "Bridge" => {
                    destroyed_container.put(&WALKABLE, false);
                }
                _ => context.state.board_mut().put_unit(position, None),
            },
            None => context.state.board_mut().put_unit(position, None),
        }
    }
}

impl RulesetProvider for DefaultV3 {
    fn action_provider(&self) -> Box<&'static dyn ActionProvider> {
        static ACTION_PROVIDER: DefaultV3Actions = DefaultV3Actions::new();
        Box::new(&ACTION_PROVIDER)
    }

    fn handle_tick(&self, state: &mut State) -> Response {
        let mut units = state.board_mut().get_all_units_mut();
        units.iter_mut().filter(|u| u.get_class() == "Tank").for_each(|u| {
            let container = u.mut_container();
            let old_actions = container.get(&ACTIONS);
            container.put(&ACTIONS, 1i32.min(old_actions.unwrap_or(&0i32) + 1i32));
        });
        Response::new_empty()
    }

    fn handle_checks(&self, _state: &State) -> Response {
        // TODO handle game end
        Response::new_empty()
    }

    fn handle_damage(&self, context: &mut Context, position: &Position) -> Response {
        let damage = context.log_entry.fields.get_or_else(&DAMAGE, 1);

        match context
            .state
            .board_mut()
            .get_highest(position)
            .expect("Cannot handle damage for invalid position")
        {
            Tile::Unit(_) => {
                let unit = context
                    .state
                    .board_mut()
                    .get_unit_mut_unsafe(position)
                    .unwrap()
                    .mut_container();
                let durability = unit
                    .get(&DURABILITY)
                    .expect("Attempted to handle damage on floor with no durability attribute");
                let new_durability = 0i32.max(durability - damage);
                unit.put(&DURABILITY, new_durability);
                if new_durability == 0i32 {
                    self.handle_destroy(context, position);
                }
            }
            Tile::Floor(_) => {
                let floor = context
                    .state
                    .board_mut()
                    .get_unit_mut_unsafe(position)
                    .unwrap()
                    .mut_container();
                let durability = floor
                    .get(&DURABILITY)
                    .expect("Attempted to handle damage on floor with no durability attribute");
                let new_durability = 0i32.max(durability - damage);
                floor.put(&DURABILITY, new_durability);
                if new_durability == 0i32 {
                    self.handle_destroy(context, position);
                }
            }
        }

        Response::new_empty()
    }

    fn handle_destroy(&self, context: &mut Context, position: &Position) -> Response {
        let tile = context
            .state
            .board_mut()
            .get_highest(position)
            .expect("Position out of bounds for handle destroy");

        match tile {
            Tile::Unit(_) => self.handle_unit_destroy(context, position),
            Tile::Floor(_) => self.handle_floor_destroy(context, position),
        }

        Response::new_empty()
    }
}
