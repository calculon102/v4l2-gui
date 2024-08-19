use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::{
    prelude::*, Application, HeaderBar, OverlaySplitView, PreferencesGroup, PreferencesPage, StatusPage
};
use aperture::{DeviceProvider, Viewfinder};
use components::create_pref_row_with_box_and_label;
use controls::{BooleanControl, ButtonControl, IntegerControl, MenuControl};
use controls::ControlUi;
use gtk::{ApplicationWindow, Orientation, Revealer, ToggleButton};
use gtk::{
    glib, Align, Label,
    ScrolledWindow,
};
use v4l::prelude::*;
use widgets::CapsPanel;

mod camera;
mod components;
mod controls;
mod files;
mod key_value_item;
mod widgets;

const APP_ID: &str = "de.pixelgerecht.v4l2_gui";

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;


// Next Steps
// TODO All controls
// TODO Hot (de-)plug?
// TODO Flatpack packaging
// TODO Error / Notice, when controls cannot be read
// TODO CLI-Param to overide /dev/video*
// TODO About Dialog
// TODO Reset to defaults
// TODO Application Icon
// TODO Collapse icon for view

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(startup);
    app.connect_activate(build_ui);

    app.run()
}

fn startup(_app: &Application) {
    aperture::init(APP_ID);
}

fn build_ui(app: &Application) {
    let device_provider = DeviceProvider::instance();
    
    // TODO Maybe in background check, with spinner in GUI?
    match device_provider.start() {
        Ok(_) => {}, // Just continue
        Err(e) => {
            present_window_with_error(
                app,
                "Error starting device provider".to_string(),
                e.to_string()
            );
            return;
        }
    };

    let default_camera = device_provider.camera(0);
    let pref_groups = match &default_camera {
        Some(c) => create_prefs_for_path(camera::get_path(&c)),
        None => {
            present_window_with_error(
                app,
                "No camera-device found".to_string(),
                "Connect a camera and restart this app".to_string()
            );
            return;
        },
    };

    let page = Rc::new(PreferencesPage::builder()
        .height_request(800)
        .hexpand(false)
        .vexpand(true)
        .width_request(400) 
        .build());
    let pref_groups_ref = Rc::new(RefCell::new(pref_groups));

    let groups: &RefCell<Vec<PreferencesGroup>> = pref_groups_ref.borrow();
    for group in groups.borrow().iter() {
        page.add(group);
    }

    let sidebar = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(600)
        .vexpand(true)
        .build();

    sidebar.set_child(Some(page.as_ref()));

    let camera_view = Rc::new(Viewfinder::new());

    let caps_panel = Rc::new(RefCell::new(CapsPanel::new(&default_camera.unwrap())));
    let caps_panel_ref: &RefCell<CapsPanel> = caps_panel.borrow();

    let caps_revealer = Revealer::builder()
        .child(caps_panel_ref.borrow().get_panel().as_ref())
        .reveal_child(false)
        .transition_type(gtk::RevealerTransitionType::SlideLeft)
        .build();

    let content = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .spacing(12)
        .build();

    content.append(camera_view.as_ref());
    content.append(&caps_revealer);

    let device_selection_box = camera::get_camera_selection_box(
        page.clone(),
        pref_groups_ref.clone(),
        camera_view.clone(),
        caps_panel.clone(),
    );

    let header_bar = HeaderBar::builder()
        .title_widget(&device_selection_box)
        .build();

    // TODO Use icon
    let caps_reveal_button = ToggleButton::builder()
        .label("Details")
        .css_classes(["flat"])
        .build();

    caps_reveal_button.connect_clicked(move |_| {
        caps_revealer.set_reveal_child(
            !caps_revealer.reveals_child()
        );
    });

    header_bar.pack_end(&caps_reveal_button);

    let split_view = OverlaySplitView::builder()
        .content(&content)
        .sidebar(&sidebar)
        .max_sidebar_width(800.0)
        .min_sidebar_width(600.0)
        .pin_sidebar(true)
        .build();

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .child(&split_view)
        .height_request(WINDOW_HEIGHT)
        .titlebar(&header_bar)
        .width_request(WINDOW_WIDTH)
        .build();

    window.present();
}

fn present_window_with_error(app: &Application, title: String, description: String) {
    // TODO Add Icon
    let status_page = StatusPage::builder()
        .title(title)
        .description(description)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&status_page)
        .height_request(WINDOW_HEIGHT)
        .title("Camera Controls")
        .width_request(WINDOW_WIDTH)
        .build();

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
    // let caps_group = create_caps_panel(device.clone());
    // groups.push(caps_group);

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

