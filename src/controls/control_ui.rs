use std::rc::Rc;

use adw::PreferencesRow;
use v4l::control::Description;

pub trait ControlUi {
    fn preference_row(&self) -> Rc<PreferencesRow>;
    fn update_value(&self, description: &Description);
    fn update_state(&self, description: &Description);
    fn reset_default(&self);
}

