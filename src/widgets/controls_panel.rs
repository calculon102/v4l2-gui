use std::{cell::RefCell, collections::HashMap, rc::Rc};

use adw::{prelude::*, PreferencesGroup};
use gtk::{Align, Label};
use v4l::Device;

use crate::{components::create_pref_row_with_box_and_label, controls::{BooleanControl, ButtonControl, ControlUi, IntegerControl, MenuControl}};

pub struct ControlsPanel {
    // WTF!? I just want to use this in a closure, re-used in event handlers of the controls
    // So,
    //  1. Rc: Shared reference
    //  2. RefCell: Mutable withon
    //  3. Hashmap: Control-UIs by Control-Id
    //  4. Rc: Shared reference of the ControlUI
    //  5. Box: In Heap, since...
    //  6. dyn ControlUi: ...it's a trait, which size is not known at build time
    control_uis: Rc<RefCell<HashMap<u32, Rc<Box<dyn ControlUi>>>>>,
    pref_groups: Vec<PreferencesGroup>,
}

impl ControlsPanel {
    pub fn new(device_path: String) -> Self {
        let device = Device::with_path(device_path);
        let control_uis: Rc<RefCell<HashMap<u32, Rc<Box<dyn ControlUi>>>>> = Rc::new(RefCell::new(HashMap::new()));

        let pref_groups = match device {
            Ok(d) => create_controls_for_device(Rc::new(d), control_uis.clone()),
            Err(e) => create_group_with_error(e.to_string()),
        };

        ControlsPanel {
            control_uis,
            pref_groups,
        }
    }

    pub fn switch_device(&mut self, device_path: String) {
        let device = Device::with_path(device_path);

        let control_uis: Rc<RefCell<HashMap<u32, Rc<Box<dyn ControlUi>>>>> = Rc::new(RefCell::new(HashMap::new()));
        self.control_uis.as_ref().borrow_mut().clear();

        let mut pref_groups = match device {
            Ok(d) => create_controls_for_device(Rc::new(d), control_uis.clone()),
            Err(e) => create_group_with_error(e.to_string()),
        };

        self.pref_groups.clear();
        self.pref_groups.append(&mut pref_groups);
    }

    pub fn reset_defaults(&self) {
        let controls = self.control_uis.as_ref().borrow();
        for control in controls.values() {
            control.reset_default();
        } 
    }

    pub fn get_pref_groups(&self) -> Vec<PreferencesGroup> {
        self.pref_groups.clone()
    }
}

fn create_controls_for_device(device: Rc<Device>, control_uis: Rc<RefCell<HashMap<u32, Rc<Box<dyn ControlUi>>>>>) -> Vec<PreferencesGroup> {
    let mut groups = vec![];

    // Closure for the change handlers to update all controls
    let device_copy = device.clone();
    let control_uis_copy = control_uis.clone();
    let update_controls_fn: Rc<Box<dyn Fn() + 'static>> = Rc::new(Box::new(move || {
        update_controls(device_copy.clone(), control_uis_copy.clone())
    }));

    // Create a group for each control class
    let ctrls_result = device.query_controls();
    if ctrls_result.is_err() {
        eprintln!(
            "Error querying controls for device: {}",
            ctrls_result.unwrap_err().to_string()
        );
        return groups;
    }

    let ctrls = ctrls_result.unwrap();

    for ctrl_desc in ctrls.iter() {
        // Ignore disabled controls
        if ctrl_desc.flags.contains(v4l::control::Flags::DISABLED) {
            println!("Ignoring disabled control {}", ctrl_desc.name);
            continue;
        }

        if groups.is_empty() && ctrl_desc.typ != v4l::control::Type::CtrlClass {
            let new_group = PreferencesGroup::builder().title("Controls").build();
            groups.push(new_group);
        }

        let ctrl_ui: Box<dyn ControlUi> = match ctrl_desc.typ {
            // TODO Implement
            v4l::control::Type::Area => {
                println!("Ignore area control {}", ctrl_desc.name);
                continue;
            },

            // Boolean-control
            v4l::control::Type::Boolean => {
                let ctrl_ui = BooleanControl::new(
                    device.clone(),
                    &ctrl_desc,
                    update_controls_fn.clone()
                );

                Box::new(ctrl_ui)
            }

            // Button with action on camera
            v4l::control::Type::Button => {
                let ctrl_ui = ButtonControl::new(
                    device.clone(),
                    &ctrl_desc,
                    update_controls_fn.clone()
                );

                Box::new(ctrl_ui)
            }

            // TODO Implement
            v4l::control::Type::Bitmask => {
                println!("Ignore Bitmask Control");
                continue;
            },

            // Control-groups
            v4l::control::Type::CtrlClass => {
                let new_group = PreferencesGroup::builder()
                    .title(ctrl_desc.name.clone())
                    .build();

                groups.push(new_group);
                continue;
            }

            // Slider-controls
            v4l::control::Type::Integer
            | v4l::control::Type::Integer64
            | v4l::control::Type::U8
            | v4l::control::Type::U16
            | v4l::control::Type::U32 => {
                let ctrl_ui = IntegerControl::new(
                    device.clone(),
                    &ctrl_desc,
                    || { }
                );

                Box::new(ctrl_ui)
            }

            v4l::control::Type::IntegerMenu | v4l::control::Type::Menu => {
                let ctrl_ui = MenuControl::new(
                    device.clone(),
                    &ctrl_desc,
                    update_controls_fn.clone()
                );

                Box::new(ctrl_ui)
            }

            // TODO Implement
            v4l::control::Type::String => {
                println!("Ignore string control");
                continue;
            },
        };

        let ctrl_ui_rc = Rc::new(ctrl_ui);
        control_uis.as_ref().borrow_mut().insert(ctrl_desc.id, ctrl_ui_rc.clone());

        let row = ctrl_ui_rc.clone().preference_row().clone();
        groups
            .last()
            .expect("No group set, while building controls")
            .add(row.as_ref());
    }

    return groups;
}


fn update_controls(device: Rc<Device>, control_uis: Rc<RefCell<HashMap<u32, Rc<Box<dyn ControlUi>>>>>) {
    let descriptions = match device.query_controls() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error querying controls for update: {}", e.to_string());
            return;
        },
    };

    let cuis_cell: &RefCell<HashMap<u32, Rc<Box<dyn ControlUi>>>> = control_uis.as_ref();
    let cuis_map = cuis_cell.borrow();
    for desc in descriptions {
        let ctrl_ui = match cuis_map.get(&desc.id) {
            Some(c) => c,
            None => continue,
        };

        ctrl_ui.update_state(&desc);
        ctrl_ui.update_value(&desc);
    }
}

fn create_group_with_error(msg: String) -> Vec<PreferencesGroup> {
    let err_group = PreferencesGroup::builder().title("Error").build();

    let (row, rowbox) = create_pref_row_with_box_and_label("Message".to_string());

    let err_label = Label::builder()
        .label(msg)
        .halign(Align::End)
        .hexpand(true)
        .xalign(1.0)
        .build();
    rowbox.append(&err_label);

    err_group.add(&row);
    return vec![err_group];
}

