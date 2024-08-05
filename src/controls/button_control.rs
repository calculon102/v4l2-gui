use std::rc::Rc;

use adw::PreferencesRow;
use adw::prelude::*;
use gtk::{Align, Button};
use v4l::{control::Description, Device};

use crate::components::create_hbox;

use super::control_ui::ControlUi;

pub struct ButtonControl {
    preference_row: Rc<PreferencesRow>,
    button: Button,
}

impl ButtonControl {
    pub fn new(device: Rc<Device>, description: &Description, on_click: fn()) -> Self {
        let readonly = description.flags.contains(v4l::control::Flags::READ_ONLY);
        let inactive = description.flags.contains(v4l::control::Flags::INACTIVE);

        let button = Button::builder()
            .halign(Align::Center)
            .label(description.name.clone())
            .sensitive(!readonly && !inactive)
            .build();

        let id_copy = description.id;
        let dev_copy = device.clone();
        button.connect_clicked(move |_| {
            // Spec says, button should set the control to activate
            // According to spec, the value itself is ignored
            let new_value = v4l::control::Value::Integer(0);
            let new_control = v4l::control::Control {
                id: id_copy,
                value: new_value,
            };
            match dev_copy.set_control(new_control) {
                Ok(_) => { on_click() }
                Err(e) => eprintln!("Error setting control: {}", e),
            };
        });

        let container = create_hbox();
        container.append(&button);

        let row = PreferencesRow::new();
        row.set_child(Some(&container));

        ButtonControl {
            preference_row: Rc::new(row),
            button,
        }
    } 
}

impl ControlUi for ButtonControl {
    fn preference_row(&self) -> Rc<PreferencesRow> {
        self.preference_row.clone()
    }

    fn update_value(&self, _description: &Description) {
        // No value to update
    }

    fn update_state(&self, description: &Description) {
        let readonly = description.flags.contains(v4l::control::Flags::READ_ONLY);
        let inactive = description.flags.contains(v4l::control::Flags::INACTIVE);

        self.button.set_sensitive(!readonly && !inactive);
    }
}

