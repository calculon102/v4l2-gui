use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::{
    prelude::*, Application, HeaderBar, PreferencesGroup, PreferencesPage
};
use aperture::DeviceProvider;
use components::{create_caps_panel, create_pref_row_with_box_and_label};
use controls::{BooleanControl, ButtonControl, IntegerControl, MenuControl};
use controls::ControlUi;
use gtk::ApplicationWindow;
use gtk::{
    glib, Align, Label,
    ScrolledWindow,
};
use v4l::prelude::*;

mod camera;
mod components;
mod controls;
mod files;
mod key_value_item;

const APP_ID: &str = "de.pixelgerecht.v4l2_gui";

// Next Steps
// TODO All controls
// TODO Hot (de-)plug?
// TODO Show image?
// TODO Flatpack packaging
// TODO Error / Notice, when controls cannot be read
// TODO CLI-Param to overide /dev/video*
// TODO Right align sliders

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(startup);
    app.connect_activate(build_ui);

    app.run()
}

fn startup(_app: &Application) {
    aperture::init(APP_ID);
}

fn build_ui(app: &Application) {
    let device_provider = DeviceProvider::instance();
    let _ = device_provider.start();

    let default_camera = device_provider.camera(0);
    let pref_groups = match default_camera {
        Some(c) => create_prefs_for_path(camera::get_path(&c)),
        None => create_group_with_error("No video device connected".to_string()),
    };

    let page = Rc::new(PreferencesPage::new());
    let pref_groups_ref = Rc::new(RefCell::new(pref_groups));

    let groups: &RefCell<Vec<PreferencesGroup>> = pref_groups_ref.borrow();
    for group in groups.borrow().iter() {
        page.add(group);
    }

    let device_selection_box = camera::get_camera_selection_box(
        page.clone(),
        pref_groups_ref.clone()
    );

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(600)
        .vexpand(true)
        .build();

    scroll.set_child(Some(page.as_ref()));

    let header_bar = HeaderBar::builder()
        .show_title(false)
        .build();
    header_bar.pack_start(&device_selection_box);

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .child(&scroll)
        .titlebar(&header_bar)
        .width_request(800)
        .height_request(600)
        .build();

    // Present window
    window.present();
}

fn create_prefs_for_path(device_path: String) -> Vec<PreferencesGroup> {
    let device = Device::with_path(device_path);

    match device {
        Ok(d) => create_controls_for_device(Rc::new(d)),
        Err(e) => create_group_with_error(e.to_string()),
    }
}

fn create_controls_for_device(device: Rc<Device>) -> Vec<PreferencesGroup> {
    let mut groups = vec![];

    // WTF!? I just want to use this in a closure, re-used in event handlers of the controls
    // So,
    //  1. Rc: Shared reference
    //  2. RefCell: Mutable withon
    //  3. Hashmap: Control-UIs by Control-Id
    //  4. Rc: Shared reference of the ControlUI
    //  5. Box: In Heap, since...
    //  6. dyn ControlUi: ...it's a trait, which size is not known at build time
    let control_uis: Rc<RefCell<HashMap<u32, Rc<Box<dyn ControlUi>>>>> = Rc::new(RefCell::new(HashMap::new()));

    // Closure for the change handlers to update all controls
    let device_copy = device.clone();
    let control_uis_copy = control_uis.clone();
    let update_controls_fn: Rc<Box<dyn Fn() + 'static>> = Rc::new(Box::new(move || {
        update_controls(device_copy.clone(), control_uis_copy.clone())
    }));

    // Create Caps-Info
    let caps_group = create_caps_panel(device.clone());
    groups.push(caps_group);

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
        control_uis.borrow_mut().insert(ctrl_desc.id, ctrl_ui_rc.clone());

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

    let cuis_cell: &RefCell<HashMap<u32, Rc<Box<dyn ControlUi>>>> = control_uis.borrow();
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

