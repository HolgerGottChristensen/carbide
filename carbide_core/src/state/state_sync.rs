use crate::state::environment::Environment;
use crate::widget::common_widget::CommonWidget;
use crate::state::global_state::GlobalState;

pub trait StateSync<S>: CommonWidget<S> where S: GlobalState {
    /// Insert local state from the widget into the environment.
    /// Return true if any of the keys from the widget was already
    /// in the local state.
    fn insert_local_state(&self, env: &mut Environment<S>);

    /// Update the state for this widget. Update both local, global and environment state
    fn update_all_widget_state(&mut self, env: &mut Environment<S>, global_state: &S);

    /// Update only the local state for the widget
    fn update_local_widget_state(&mut self, env: &Environment<S>);

    /*/// When implementing this, all states that are a function of globalState needs to be updated
    /// This is done by calling either get_value or get_value_mut.
    /// Todo: Update this to happen automatically
    /// You also need to update all the local states, with the values from the states list.
    fn update_widget_state(&mut self, env: &mut Environment, global_state: &S);
    */

    /// This should be implemented to synchronize both global and local state.
    /// A general implementation should:
    /// - Update the widget state, both global and local
    /// - Insert its own local state into the environment
    /// - Iterate though its children and sync_state on each
    /// You can in most cases use default_sync_state
    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S);

    fn default_sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        self.update_all_widget_state(env, global_state);

        self.insert_local_state(env);

        for child in self.get_proxied_children() {
            child.sync_state(env, global_state)
        }

        self.update_local_widget_state(env);
    }
}

pub trait NoLocalStateSync {}

impl<S: GlobalState, T> StateSync<S> for T where T: NoLocalStateSync + CommonWidget<S> {
    fn insert_local_state(&self, _: &mut Environment<S>) {}

    fn update_all_widget_state(&mut self, _: &mut Environment<S>, _: &S) {}

    fn update_local_widget_state(&mut self, _env: &Environment<S>) {}

    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        self.default_sync_state(env, global_state);
    }
}