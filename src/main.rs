use adw::{prelude::*, Application, PreferencesGroup, PreferencesPage, PreferencesRow};
use gtk::{glib, Adjustment, Box, DropDown, Label, Orientation, Scale, StringList};
use gtk::ApplicationWindow;

mod files;

const APP_ID: &str = "de.pixelgerecht.v4l2_gui";

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

    let (ctrl_row, ctrl_box) = create_pref_row_with_box_and_label("Some attributes".to_string());

    let scale = Scale::new(
        Orientation::Horizontal,
        Some(&Adjustment::new(128.0, 0.0, 257.0, 1.0, 1.0, 10.0))
    );
    scale.set_hexpand(true);

    ctrl_box.append(&scale);
    device_controls_group.add(&ctrl_row);

    let page = PreferencesPage::new();
    page.add(&device_selection_group);
    page.add(&device_controls_group);

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        // TODO Only one child
        .child(&page)
        .build();

    // Present window
    window.present();
}

/// Create a pre-configured row for a preference, that can be added to a
/// PreferencesPage
///
/// The Box already contains a label, with the given string as text.
/// Append to the box as needed.
fn create_pref_row_with_box_and_label(label: String) -> (PreferencesRow, Box) {
    let row = PreferencesRow::new();
    
    let label = Label::builder()
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .label(label.as_str())
        .use_underline(true)
        .xalign(0.0)
        .build();

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
