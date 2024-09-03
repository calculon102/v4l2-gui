use std::rc::Rc;

use adw::{prelude::*, PreferencesGroup, PreferencesPage};
use aperture::Camera;
use v4l::{format::Description, video::{capture::Parameters, Capture}, Device};

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
        let caps_group = PreferencesGroup::builder().title("About").build();

        page.add(&caps_group);

        let device_path = get_path(&for_camera);
        let device = match Device::with_path(device_path) {
            Ok(d) => d,
            Err(e) => {
                let error_msg = create_info_row(
                    "Error getting device for capabilities".to_string(),
                    e.to_string(),
                );
                caps_group.add(&error_msg);
                groups.push(caps_group);
                return;
            }
        };

        let caps = match device.query_caps() {
            Ok(caps) => caps,
            Err(e) => {
                let error_msg =
                    create_info_row("Error querying capabilities".to_string(), e.to_string());
                caps_group.add(&error_msg);
                groups.push(caps_group);
                return;
            }
        };

        let (major, minor, patch) = caps.version;
        let version_string = format!("{}.{}.{}", major, minor, patch);
        caps_group.add(&create_info_row("Bus".to_string(), caps.bus.clone()));
        caps_group.add(&create_info_row("Card".to_string(), caps.card.clone()));
        caps_group.add(&create_info_row("Driver".to_string(), caps.driver.clone()));
        caps_group.add(&create_info_row("Version".to_string(), version_string));
        caps_group.add(&create_info_row(
            "Capabilities".to_string(),
            caps.capabilities.to_string(),
        ));

        groups.push(caps_group);

        match device.params() {
            Ok(params) => {
                let group = CapsPanel::create_params_group(params);
                page.add(&group);
                groups.push(group);
            },
            Err(e) => {
                let error_msg =
                    create_info_row("Error querying params".to_string(), e.to_string());
                let panel = PreferencesGroup::builder().title("Parameters").build();
                panel.add(&error_msg);
                groups.push(panel);
            }
        };

        match device.enum_formats() {
            Ok(formats) => {
                let group = CapsPanel::create_formats_group(formats);
                page.add(&group);
                groups.push(group);
            }
            Err(e) => {
                let error_msg =
                    create_info_row("Error querying formats".to_string(), e.to_string());
                let panel = PreferencesGroup::builder().title("Formats").build();
                panel.add(&error_msg);
                groups.push(panel);
            }
        };
    }

    fn create_params_group(params: Parameters) -> PreferencesGroup {
        let group = PreferencesGroup::builder().title("Parameters").build();

        group.add(&create_info_row("Capabilities".to_string(), params.capabilities.to_string()));
        group.add(&create_info_row("Interval".to_string(), params.interval.to_string()));
        group.add(&create_info_row("Modes".to_string(), params.modes.to_string()));

        group
    }

    fn create_formats_group(descriptions: Vec<Description>) -> PreferencesGroup {
        let group = PreferencesGroup::builder().title("Formats").build();

        for desc in descriptions {
            let format = format!("FourCC: {}\nFlags: {}", desc.fourcc, desc.flags);
            group.add(&create_info_row(desc.description, format));
        }

        group
    }
}
