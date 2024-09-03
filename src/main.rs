use gtk::prelude::*;
use gtk::gio;

use application::Application;

mod application;
mod camera;
mod components;
mod controls;
mod files;
mod key_value_item;
mod widgets;

// Next Steps
// TODO Application Icon
// TODO Read all informations into capabilities
// TODO What if device is in use by other app?
// TODO What if aperture assertion?
// TODO All controls
// TODO Hot (de-)plug?
// TODO Flatpack packaging
// TODO Error / Notice, when controls cannot be read
// TODO CLI-Param to overide /dev/video*
// TODO About Dialog
fn main() -> glib::ExitCode {
    gio::resources_register_include!("camera_settings.gresource")
        .expect("Failed to register resources.");

    let app = crate::Application::new();
    app.run()
}


