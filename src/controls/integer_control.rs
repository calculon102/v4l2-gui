use std::rc::Rc;

use adw::PreferencesRow;
use adw::prelude::*;
use gtk::Adjustment;
use gtk::Orientation;
use gtk::PositionType;
use gtk::Scale;
use v4l::{control::Description, Device};

use crate::components::create_pref_row_with_box_and_label;

use super::{ControlUi, ControlValueError};

pub struct IntegerControl {
    default: i64,
    device: Rc<Device>,
    pref_row: Rc<PreferencesRow>,
    scale: Scale,
}

impl IntegerControl {
    pub fn new(device: Rc<Device>, description: &Description, on_change: fn()) -> Self {
        let readonly = description.flags.contains(v4l::control::Flags::READ_ONLY);
        let inactive = description.flags.contains(v4l::control::Flags::INACTIVE);

        let value = IntegerControl::query_state(device.as_ref(), &description);

        let adjustment = Adjustment::builder()
            .lower(description.minimum as f64)
            .upper(description.maximum as f64)
            .step_increment(description.step as f64)
            .value(value as f64)
            .build();

        let scale = Scale::builder()
            .adjustment(&adjustment)
            .digits(0)
            .draw_value(true)
            .halign(gtk::Align::End)
            .hexpand(true)
            .orientation(Orientation::Horizontal)
            .sensitive(!readonly && !inactive)
            .show_fill_level(true)
            .value_pos(PositionType::Right)
            .width_request(360)
            .build();

        // TODO Adding a mark leads to assertion failure in GTK-GSK
        // scale.add_mark(description.default as f64, PositionType::Bottom, None);

        let id_copy = description.id;
        let dev_copy = device.clone();
        scale.connect_value_changed(move |scale| {
            let new_value = v4l::control::Value::Integer(scale.value() as i64);
            let new_control = v4l::control::Control {
                id: id_copy,
                value: new_value,
            };
            match dev_copy.set_control(new_control) {
                Ok(_) => { on_change() }
                Err(e) => eprintln!("Error setting control: {}", e),
            };
        });

        let (row, rowbox) = create_pref_row_with_box_and_label(description.name.clone());
        rowbox.append(&scale);

        IntegerControl {
            default: description.default,
            device: device.clone(),
            pref_row: Rc::new(row),
            scale,
        }
    } 

    fn query_control_integer(
        device: &Device,
        ctrl_desc: &Description,
    ) -> Result<i64, ControlValueError> {
        let control = match device.control(ctrl_desc.id) {
            Ok(v) => v,
            Err(e) => return Err(ControlValueError::new(e.to_string())),
        };

        return match control.value {
            v4l::control::Value::Integer(int_val) => Ok(int_val),
            _ => Err(ControlValueError::new(format!(
                "Value of {} is not an integer",
                ctrl_desc.name
            ))),
        };
    }

    fn query_state(device: &Device, description: &Description) -> i64 {
        match IntegerControl::query_control_integer(&device, &description) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error while checking state of control: {}", e.message);
                description.default
            }
        }
    }
}

impl ControlUi for IntegerControl {
    fn preference_row(&self) -> Rc<PreferencesRow> {
        self.pref_row.clone()
    }

    fn update_value(&self, description: &Description) {
        let old_value = self.scale.value();
        let new_value = IntegerControl::query_state(self.device.as_ref(), description) as f64;

        if new_value != old_value {
            self.scale.set_value(new_value as f64);
        }
    }

    fn update_state(&self, description: &Description) {
        let readonly = description.flags.contains(v4l::control::Flags::READ_ONLY);
        let inactive = description.flags.contains(v4l::control::Flags::INACTIVE);

        self.scale.set_sensitive(!readonly && !inactive);
    }

    fn reset_default(&self) {
        self.scale.set_value(self.default as f64)
    }
}

