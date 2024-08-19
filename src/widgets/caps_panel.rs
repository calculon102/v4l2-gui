use std::rc::Rc;

use adw::{prelude::*, PreferencesGroup, PreferencesPage};
use aperture::Camera;
use v4l::Device;

use crate::{camera::get_path, components::create_info_row};

pub struct CapsPanel {
    page: Rc<PreferencesPage>,
    groups: Vec<PreferencesGroup>,
}

impl CapsPanel {
    pub fn new(for_camera: &Camera) -> Self {
        let page = PreferencesPage::builder().width_request(300).build();

        let mut groups: Vec<PreferencesGroup> = vec![];

        CapsPanel::fill_page(&page, &mut groups, for_camera);

        CapsPanel {
            page: Rc::new(page),
            groups,
        }
    }

    pub fn update(&mut self, for_camera: &Camera) {
        for group in &self.groups {
            self.page.remove(group);
        }

        let _ = &self.groups.clear();

        CapsPanel::fill_page(&self.page, &mut self.groups, for_camera);
    }

    pub fn get_panel(&self) -> Rc<PreferencesPage> {
        self.page.clone()
    }

    fn fill_page(page: &PreferencesPage, groups: &mut Vec<PreferencesGroup>, for_camera: &Camera) {
        let panel = PreferencesGroup::builder().title("About").build();

        page.add(&panel);

        let device_path = get_path(&for_camera);
        let device = match Device::with_path(device_path) {
            Ok(d) => d,
            Err(e) => {
                let error_msg = create_info_row(
                    "Error getting device for capabilities".to_string(),
                    e.to_string(),
                );
                panel.add(&error_msg);
                groups.push(panel);
                return;
            }
        };

        let caps = match device.query_caps() {
            Ok(caps) => caps,
            Err(e) => {
                let error_msg =
                    create_info_row("Error querying capabilities".to_string(), e.to_string());
                panel.add(&error_msg);
                groups.push(panel);
                return;
            }
        };

        let (major, minor, patch) = caps.version;
        let version_string = format!("{}.{}.{}", major, minor, patch);
        panel.add(&create_info_row("Bus".to_string(), caps.bus.clone()));
        panel.add(&create_info_row("Card".to_string(), caps.card.clone()));
        panel.add(&create_info_row("Driver".to_string(), caps.driver.clone()));
        panel.add(&create_info_row("Version".to_string(), version_string));
        panel.add(&create_info_row(
            "Capabilities".to_string(),
            caps.capabilities.to_string(),
        ));

        groups.push(panel);
    }
}
