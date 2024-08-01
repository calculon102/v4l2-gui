
use std::rc::Rc;

use adw::{prelude::*, Application, PreferencesGroup, PreferencesPage, PreferencesRow};
use components::{create_hbox, create_info_row, create_pref_row_with_box_and_label};
use gtk::{glib, Adjustment, Align, Button, DropDown, Label, Orientation, PositionType, Scale, ScrolledWindow, StringList};
use gtk::ApplicationWindow;
use v4l::prelude::*;

mod components;
mod files;

const APP_ID: &str = "de.pixelgerecht.v4l2_gui";

// Next Steps
// TODO Label with min width
// TODO All controls
// TODO Reload labels on change
// TODO Hot (de-)plug?
// TODO Show image?

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    let device_selection_group = PreferencesGroup::new();
    device_selection_group.set_title("Camera");

    let device_controls_group = PreferencesGroup::new();
    device_controls_group.set_title("Camera attributes");
    
    let (device_selection_row, device_selection_box) = create_pref_row_with_box_and_label("Select device".to_string());

    // Create combobox with video-devices for selection
    // TODO Error / Notice, when no camera present
    // TODO Add signal to selection and recreate attributes panel with attributes
    // TODO Print Infos about camera on selection
    let device_selection_strings = files::get_video_devices("/dev");
    let device_selection_str: Vec<&str> = device_selection_strings.iter().map(|s| s.as_str()).collect();
    let device_selection_model = StringList::new(&device_selection_str);
    let device_selection_dropdown = DropDown::builder()
        .hexpand(true)
        .model(&device_selection_model)
        .build();

    device_selection_box.append(&device_selection_dropdown);
    device_selection_group.add(&device_selection_row);

    let page = PreferencesPage::new();
    page.add(&device_selection_group);

    let pref_groups: Vec<PreferencesGroup> = create_prefs_for_path("/dev/video0".to_string());
    for group in pref_groups.iter() {
        page.add(group);
    }

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(600)
        .vexpand(true)
        .build();

    scroll.set_child(Some(&page));

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .child(&scroll)
        .title("Camera Controls")
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

    // Create Caps-Info
    let caps_result = device.query_caps();

    if caps_result.is_err() {
        eprintln!("Error querying caps for device: {}", caps_result.unwrap_err().to_string());
        return groups;
    }

    let caps = caps_result.unwrap();

    let info_group = PreferencesGroup::builder()
        .title("Information")
        .build();

    let (major, minor, patch) = caps.version;
    let version_string = format!("{}.{}.{}", major, minor, patch);
    info_group.add(&create_info_row("Bus".to_string(), caps.bus.clone()));
    info_group.add(&create_info_row("Card".to_string(), caps.card.clone()));
    info_group.add(&create_info_row("Driver".to_string(), caps.driver.clone()));
    info_group.add(&create_info_row("Version".to_string(), version_string));
    info_group.add(&create_info_row("Capabilities".to_string(), caps.capabilities.to_string()));

    groups.push(info_group);


    // Create a group for each control class
    let ctrls_result = device.query_controls();
    if ctrls_result.is_err() {
        eprintln!("Error querying controls for device: {}", ctrls_result.unwrap_err().to_string());
        return groups;
    }

    let ctrls = ctrls_result.unwrap();

    // TODO Handle controls with UPDATE flag
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

        let readonly = ctrl_desc.flags.contains(v4l::control::Flags::READ_ONLY);

        match ctrl_desc.typ {
            // TODO Implement
            v4l::control::Type::Area => println!("Ignore area control {}", ctrl_desc.name),

            // Button with action on camera 
            v4l::control::Type::Button => {
                let button = Button::builder()
                    .halign(Align::Center)
                    .label(ctrl_desc.name.clone())
                    .build();

                let id_copy = ctrl_desc.id;
                let dev_copy = device.clone();
                button.connect_clicked(move |_| {
                    // Spec says, button should set the control to activate
                    // According to spec, the value itself is ignored
                    let new_value = v4l::control::Value::Integer(0);
                    let new_control = v4l::control::Control { id: id_copy, value: new_value };
                    match dev_copy.set_control(new_control) {
                        Ok(_) => {},
                        Err(e) => eprintln!("Error setting control: {}", e),
                    };
                });

                let container = create_hbox();
                container.append(&button);

                let row = PreferencesRow::new();
                row.set_child(Some(&container));

                groups.last().expect("No group set, while building controls").add(&row);
            },

            // TODO Implement
            v4l::control::Type::Boolean => {
                 
            },

            // TODO Implement
            v4l::control::Type::Bitmask => println!("TODO Bitmask Control"),

            // Control-groups
            v4l::control::Type::CtrlClass => {
                let new_group = PreferencesGroup::builder()
                    .title(ctrl_desc.name.clone())
                    .build();

                groups.push(new_group);
            },

            // Slider-controls
            v4l::control::Type::Integer |
            v4l::control::Type::Integer64 | 
            v4l::control::Type::U8 |
            v4l::control::Type::U16 |
            v4l::control::Type::U32 => {
                let control = device.control(ctrl_desc.id);

                if control.is_err() {
                    eprintln!("Error reading control {}: {}", ctrl_desc.name, control.unwrap_err().to_string());
                    continue;
                }

                let control_value = control.unwrap().value;
                let extracted_value = match control_value {
                    v4l::control::Value::Integer(int) => int,
                    _ => {
                        eprintln!("Expected Integer for {}, but got ", ctrl_desc.name);
                        // TODO better way?
                        dbg!(control_value);
                        continue;
                    }
                };

                let adjustment = Adjustment::builder()
                    .lower(ctrl_desc.minimum as f64)
                    .upper(ctrl_desc.maximum as f64)
                    .step_increment(ctrl_desc.step as f64)
                    .value(extracted_value as f64)
                    .build();

                let scale = Scale::builder()
                    .adjustment(&adjustment)
                    .digits(0)
                    .draw_value(true)
                    .hexpand(true)
                    .orientation(Orientation::Horizontal)
                    .sensitive(!readonly)
                    .show_fill_level(true)
                    .value_pos(PositionType::Right)
                    .build();

                scale.add_mark(ctrl_desc.default as f64, PositionType::Bottom, None);

                let id_copy = ctrl_desc.id;
                let dev_copy = device.clone();
                scale.connect_value_changed(move |scale| {
                    let new_value = v4l::control::Value::Integer(scale.value() as i64);
                    let new_control = v4l::control::Control { id: id_copy, value: new_value };
                    match dev_copy.set_control(new_control) {
                        Ok(_) => {},
                        Err(e) => eprintln!("Error setting control: {}", e),
                    };
                });

                let (row, rowbox) = create_pref_row_with_box_and_label(ctrl_desc.name.clone());
                rowbox.append(&scale);

                groups.last().expect("No group set, while building controls").add(&row);
            }

            v4l::control::Type::IntegerMenu |
            v4l::control::Type::Menu => println!("TODO Menu"),
            v4l::control::Type::String => println!("TODO String"),
        }
    }

    return groups;
} 

fn create_group_with_error(msg: String) -> Vec<PreferencesGroup> {
    let err_group = PreferencesGroup::builder()
        .title("Error")
        .build();

    let (row, rowbox) = create_pref_row_with_box_and_label("Message".to_string());

    let err_label = Label::builder()
        .label(msg)
        .build();
    rowbox.append(&err_label);

    err_group.add(&row);

    return vec![err_group];
}
