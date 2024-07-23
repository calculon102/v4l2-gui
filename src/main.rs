use gtk::{glib, Application, DropDown, StringList};
use gtk::{prelude::*, ApplicationWindow};

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
    // Create combobox with video-devices for selection
    let video_devices_strings = files::get_video_devices("/dev");
    let video_devices_str: Vec<&str> = video_devices_strings.iter().map(|s| s.as_str()).collect();
    let video_devices_list = StringList::new(&video_devices_str);
    let video_devices_dropdown = DropDown::builder()
        .margin_bottom(12)
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .model(&video_devices_list)
        .build();


    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&video_devices_dropdown)
        .build();

    // Present window
    window.present();
}
