
use adw::{prelude::*, ActionRow, Application, PreferencesGroup, PreferencesPage, PreferencesRow};
use gtk::{glib, Adjustment, Box, DropDown, Label, Orientation, PositionType, Scale, StringList};
use gtk::ApplicationWindow;
use v4l::prelude::*;

mod files;

const APP_ID: &str = "de.pixelgerecht.v4l2_gui";

// Next Steps
// TODO Scrolling
// TODO Bigger initial window
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

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .child(&page)
        .title("My GTK App")
        .width_request(800)
        .height_request(600)
        .build();

    // Present window
    window.present();
}

fn create_label(label: String) -> Label {
    Label::builder()
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .label(label.as_str())
        .use_underline(true)
        .xalign(0.0)
        .build()
}


/// Create a pre-configured row for a preference, that can be added to a
/// PreferencesPage
///
/// The Box already contains a label, with the given string as text.
/// Append to the box as needed.
fn create_pref_row_with_box_and_label(label: String) -> (PreferencesRow, Box) {
    let row = PreferencesRow::new();
    
    let label = create_label(label);

    let rowbox = Box::builder()
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    rowbox.append(&label);
    row.set_child(Some(&rowbox));

    (row, rowbox)
}

fn create_info_row(label: String, info: String) -> ActionRow {
    let action_row = ActionRow::builder()
        .title(label)
        .subtitle(info)
        .subtitle_selectable(true)
        .build();

//    let (row, rowbox) = create_pref_row_with_box_and_label(label);

//    let info_label = create_info(info);
//    rowbox.append(&info_label);

    return action_row;
}

fn create_prefs_for_path(device_path: String) -> Vec<PreferencesGroup> {
    let device = Device::with_path(device_path);

    match device {
        Ok(d) => create_controls_for_device(d),
        Err(e) => create_group_with_error(e.to_string()),
    }
}

fn create_controls_for_device(device: Device) -> Vec<PreferencesGroup> {
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

    // Default group if there is not group defined in the controls of the camera
    let mut current_group = PreferencesGroup::builder()
        .title("Controls")
        .build();

    for ctrl_desc in ctrls.iter() {
        match ctrl_desc.typ {
            v4l::control::Type::Area => println!(""),
            v4l::control::Type::Button => println!(""),
            v4l::control::Type::Boolean => println!(""),
            v4l::control::Type::Bitmask => println!(""),

            // Control-groups
            v4l::control::Type::CtrlClass => {
                // TODO Does not work
                // Only use current group, it contains controls - especially the default group
                if current_group.first_child().is_some() {
                    groups.push(current_group);
                }

                current_group = PreferencesGroup::builder()
                    .title(ctrl_desc.name.clone())
                    .build();
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

                // TODO Add event handler to set control
                let scale = Scale::builder()
                    .adjustment(&adjustment)
                    .draw_value(true)
                    .hexpand(true)
                    .orientation(Orientation::Horizontal)
                    .show_fill_level(true)
                    .value_pos(PositionType::Right)
                    .build();

                let (row, rowbox) = create_pref_row_with_box_and_label(ctrl_desc.name.clone());
                rowbox.append(&scale);

                current_group.add(&row);
            }

            v4l::control::Type::IntegerMenu => println!(""),
            v4l::control::Type::Menu => println!(""),
            v4l::control::Type::String => println!(""),
        }
    }
//
//
//    let (ctrl_row, ctrl_box) = create_pref_row_with_box_and_label("Some attributes".to_string());
//
//    let scale = Scale::new(
//        Orientation::Horizontal,
//        Some(&Adjustment::new(128.0, 0.0, 257.0, 1.0, 1.0, 10.0))
//    );
//    scale.set_hexpand(true);
//
//    ctrl_box.append(&scale);
//    group.add(&ctrl_row);

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
