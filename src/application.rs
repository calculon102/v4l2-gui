use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use adw::{
    prelude::*, HeaderBar, OverlaySplitView, PreferencesPage, StatusPage
};
use aperture::{DeviceProvider, Viewfinder};
use gtk::{ApplicationWindow, Button, Orientation, Revealer, ToggleButton};
use gtk::{
    gio, glib,
    ScrolledWindow,
};
use crate::widgets::CapsPanel;
use log::debug;

const APP_ID: &str = "de.pixelgerecht.CameraSettings";

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

mod imp {
    use crate::widgets::ControlsPanel;

    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Debug, Default)]
    pub struct Application;
    
    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            debug!("Application::activate");

            self.parent_activate();
            let app = self.obj();

            if let Some(window) = app.active_window() {
                window.present();
                return;
            }

            let device_provider = DeviceProvider::instance();

            // TODO Maybe in background check, with spinner in GUI?
            match device_provider.start() {
                Ok(_) => {}, // Just continue
                Err(e) => {
                    present_window_with_error(
                        app.as_ref(),
                        "Error starting device provider".to_string(),
                        e.to_string()
                    );
                    return;
                }
            };

            let page = Rc::new(PreferencesPage::builder()
                .height_request(800)
                .hexpand(false)
                .vexpand(true)
                .width_request(400) 
                .build());

            let default_camera = device_provider.camera(0);

            let controls_panel = match &default_camera {
                Some(c) => {
                    let cp = ControlsPanel::new(crate::camera::get_path(&c));
                    Rc::new(RefCell::new(cp))
                },
                None => {
                    present_window_with_error(
                        app.as_ref(),
                        "No camera-device found".to_string(),
                        "Connect a camera and restart this app".to_string()
                    );
                    return;
                },
            };

            let groups: &RefCell<ControlsPanel> = controls_panel.borrow();
            for group in groups.borrow().get_pref_groups().iter() {
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

            let device_selection_box = crate::camera::get_camera_selection_box(
                controls_panel.clone(),
                camera_view.clone(),
                caps_panel.clone(),
            );

            let header_bar = HeaderBar::builder()
                .title_widget(&device_selection_box)
                .build();

            let reset_defaults_button = Button::builder()
                .css_classes(["flat"])
                .icon_name("arrow-hook-left-horizontal2-symbolic")
                .tooltip_text("Reset camera defaults")
                .build();

            let controls_panel_for_reset = controls_panel.clone();
            reset_defaults_button.connect_clicked(move |_| {
                controls_panel_for_reset
                    .as_ref()
                    .borrow()
                    .reset_defaults();
            });

            let caps_reveal_button = ToggleButton::builder()
                .css_classes(["flat"])
                .icon_name("info-outline-symbolic")
                .tooltip_text("Show camera details")
                .build();

            caps_reveal_button.connect_clicked(move |_| {
                caps_revealer.set_reveal_child(
                    !caps_revealer.reveals_child()
                );
            });

            header_bar.pack_start(&reset_defaults_button);
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
                .application(app.as_ref())
                .child(&split_view)
                .height_request(WINDOW_HEIGHT)
                .titlebar(&header_bar)
                .width_request(WINDOW_WIDTH)
                .build();

            window.present();
        }

        fn startup(&self) {
            self.parent_startup();
            aperture::init(APP_ID);

            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_gactions();
            app.setup_accels();
        }
    }

    impl GtkApplicationImpl for Application {}

    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/de/pixelgerecht/CameraSettings/")
            .build()
    }
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }

    fn setup_gactions(&self) {
        let actions = [gio::ActionEntryBuilder::new("quit")
            .activate(|app: &Self, _, _| app.quit())
            .build()];
        self.add_action_entries(actions);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Ctrl>w"]);
    }
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
