use crate::rule::action::ActionProvider;
use crate::rule::context::Context;
use crate::ruleset::response::Response;
use crate::state::position::Position;
use crate::state::state::State;

pub trait RulesetProvider {
    fn action_provider(&self) -> Box<&'static dyn ActionProvider>;
    fn handle_tick(&self, state: &mut State) -> Response;
    fn handle_checks(&self, state: &State) -> Response;
    fn handle_damage(&self, context: &mut Context, position: &Position) -> Response;
    fn handle_destroy(&self, context: &mut Context, position: &Position) -> Response;
}

pub struct Ruleset {
    pub action_provider: Box<&'static dyn ActionProvider>,
    pub handle_tick: Box<dyn Fn(&mut State) -> Response>,
    pub handle_checks: Box<dyn Fn(&mut State) -> Response>,
    pub handle_damage: Box<dyn Fn(&mut Context, &Position) -> Response>,
    pub handle_destroy: Box<dyn Fn(&mut Context, &Position) -> Response>,
}

impl Ruleset {
    pub fn new(provider: Box<&'static dyn RulesetProvider>) -> Ruleset {
        Ruleset {
            action_provider: provider.action_provider(),
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
}
