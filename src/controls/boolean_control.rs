use std::rc::Rc;

use adw::{PreferencesRow, SwitchRow};
use adw::prelude::*;
use glib::subclass::shared::RefCounted;
use v4l::{control::Description, Device};

use super::{ControlUi, ControlValueError};

pub struct BooleanControl {
    device: Rc<Device>,
    switch_row: Rc<SwitchRow>,
}

impl BooleanControl {
    pub fn new(device: Rc<Device>, description: &Description, on_switch: fn()) -> Self {
        let readonly = description.flags.contains(v4l::control::Flags::READ_ONLY);
        let inactive = description.flags.contains(v4l::control::Flags::INACTIVE);

        let active = BooleanControl::query_state(device.as_ref(), description);

        let row = SwitchRow::builder()
            .active(active)
            .hexpand(true)
            .sensitive(!readonly && !inactive)
            .title(description.name.clone())
            .build();

        let id_copy =description.id;
        let dev_copy = device.clone();
        row.connect_active_notify(move |row| {
            let new_value = v4l::control::Value::Boolean(row.is_active());
            let new_control = v4l::control::Control {
                id: id_copy,
                value: new_value,
            };
            match dev_copy.set_control(new_control) {
                Ok(_) => { on_switch() }
                Err(e) => eprintln!("Error setting control: {}", e),
            };
        });

        BooleanControl {
            device: device.clone(),
            switch_row: Rc::new(row),
        }
    } 

    fn query_control_boolean(
        device: &Device,
        ctrl_desc: &Description,
    ) -> Result<bool, ControlValueError> {
        let control = match device.control(ctrl_desc.id) {
            Ok(v) => v,
            Err(e) => return Err(ControlValueError::new(e.to_string())),
        };

        return match control.value {
            v4l::control::Value::Boolean(bool_val) => Ok(bool_val),
            _ => Err(ControlValueError::new(format!(
                "Value of {} is not a boolean",
                ctrl_desc.name
            ))),
        };
    }

    fn query_state(device: &Device, description: &Description) -> bool {
        match BooleanControl::query_control_boolean(&device, &description) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error while checking state of control: {}", e.message);
                false
            }
        }
    }
}

impl ControlUi for BooleanControl {
    fn preference_row(&self) -> Rc<PreferencesRow> {
        // TODO Safe method
        let row = unsafe {
            let ptr = self.switch_row.clone().into_raw();
            Rc::<PreferencesRow>::from_raw(ptr.cast())
        };
        row
    }

    fn update_value(&self, description: &Description) {
        let active = BooleanControl::query_state(self.device.as_ref(), description);

        self.switch_row.set_active(active);
    }

    fn update_state(&self, description: &Description) {
        let readonly = description.flags.contains(v4l::control::Flags::READ_ONLY);
        let inactive = description.flags.contains(v4l::control::Flags::INACTIVE);

        self.switch_row.set_sensitive(!readonly && !inactive);
    }
}

