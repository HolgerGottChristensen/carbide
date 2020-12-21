use state::state::LocalStateList;
use state::environment::Environment;
use widget::common_widget::CommonWidget;

pub trait StateSync<S> {

    /// Insert local state from the widget into the environment.
    /// Return true if any of the keys from the widget was already
    /// in the local state.
    fn insert_local_state(&self, env: &mut Environment) -> bool;

    fn remove_local_state(&self, env: &mut Environment);

    fn replace_local_state(&self, env: &mut Environment);

    fn update_widget_state(&mut self, env: &Environment, global_state: &S);


    /*/// When implementing this, all states that are a function of globalState needs to be updated
    /// This is done by calling either get_value or get_value_mut.
    /// Todo: Update this to happen automatically
    /// You also need to update all the local states, with the values from the states list.
    fn update_widget_state(&mut self, env: &mut Environment, global_state: &S);
    */

    fn sync_state(&mut self, env: &mut Environment, global_state: &S);


}

pub trait NoLocalStateSync {}

impl<S, T> StateSync<S> for T where T: NoLocalStateSync + CommonWidget<S> {
    fn insert_local_state(&self, _: &mut Environment) -> bool {
        false
    }

    fn pop_local_state(&self, _: &mut Environment) {}

    fn replace_local_state(&self, _: &mut Environment) {}

    fn update_widget_state(&mut self, _: &Environment, _: &S) {}

    fn sync_state(&mut self, env: &mut Environment, global_state: &S) {
        self.update_widget_state(env, global_state);
        self.push_local_state(env);

        for child in self.get_children_mut() {
            child.sync_state(env, global_state)
        }

        self.pop_local_state(env)
    }
}