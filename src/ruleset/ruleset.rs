use crate::rule::action::{ActionDescription, ActionProvider};
use crate::rule::context::Context;
use crate::rule::log_entry::EntryData;
use crate::ruleset::response::Response;
use crate::state::position::Position;
use crate::state::state::State;
use std::collections::HashMap;

pub trait RulesetProvider {
    fn action_provider(&self) -> Box<&'static dyn ActionProvider>;
    fn handle_tick(&self, state: &mut State) -> Response;
    fn handle_checks(&self, state: &mut State) -> Response;
    fn handle_damage(&self, context: &mut Context, position: &Position) -> Response;
    fn handle_destroy(&self, context: &mut Context, position: &Position) -> Response;
}

pub struct Ruleset {
    action_map: HashMap<String, ActionDescription>,
    handle_tick: Box<dyn Fn(&mut State) -> Response>,
    handle_checks: Box<dyn Fn(&mut State) -> Response>,
    handle_damage: Box<dyn Fn(&mut Context, &Position) -> Response>,
    handle_destroy: Box<dyn Fn(&mut Context, &Position) -> Response>,
}

impl Ruleset {
    pub fn new(provider: Box<&'static dyn RulesetProvider>) -> Ruleset {
        Ruleset {
            action_map: provider.action_provider().actions(),
            handle_tick: Box::new(|state| -> Response { provider.handle_tick(state) }),
            handle_checks: Box::new(|state| -> Response { provider.handle_checks(state) }),
            handle_damage: Box::new(|context, position| -> Response {
                provider.handle_damage(context, position)
            }),
            handle_destroy: Box::new(|context, position| -> Response {
                provider.handle_destroy(context, position)
            }),
        }
    }

    fn handle_tick(&self, state: &mut State) -> Response {
        let mut response = (self.handle_tick)(state);
        response.concat((self.handle_checks)(state));
        response
    }

    pub fn handle_action(&self, context: &mut Context) -> Response {
        match &context.log_entry.entry_type {
            EntryData::StartOfDay => self.handle_tick(context.state),
            EntryData::PlayerAction(action_name) => {
                self.action_map
                    .get(action_name)
                    .expect(format!("Action {} is not implemented", action_name).as_str())
                    .apply(context);
                (self.handle_checks)(context.state)
            }
        }
    }
}
