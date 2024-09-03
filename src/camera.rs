use std::{cell::RefCell, rc::Rc};

use aperture::{Camera, DeviceProvider, Viewfinder};
use gtk::{Align, Box, DropDown, Label, ListItem, SignalListItemFactory};
use adw::prelude::*;

use crate::{components::create_hbox, widgets::{CapsPanel, ControlsPanel}};


pub fn get_camera_selection_box(
    pref_groups: Rc<RefCell<ControlsPanel>>,
    camera_view: Rc<Viewfinder>,
    caps_panel: Rc<RefCell<CapsPanel>>,
    ) -> Box {
    let device_selection_box = create_hbox();

    let device_provider = DeviceProvider::instance();
    let _ = device_provider.start();
    
    let camera_count = device_provider.n_items();
    
    if camera_count == 0 {
        let label = Label::builder()
            .label("No camera detected")
            .halign(Align::Center)
            .xalign(0.5)
            .build();

        device_selection_box.append(&label);
        return device_selection_box;
    }

    if camera_count == 1 {
        let only_cam = device_provider.camera(0).unwrap();
        let camera_label = get_name(&only_cam);

        let label = Label::builder()
            .label(&camera_label)
            .halign(Align::Center)
            .xalign(0.5)
            .build();

        device_selection_box.append(&label);
        return device_selection_box;
    }

    let factory = SignalListItemFactory::new();

    factory.connect_setup(|_, list_item| {
        let label = Label::builder()
            .halign(Align::Start)
            .build();
        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&label));
    });

    factory.connect_bind(|_, list_item| {
        let cam = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .item()
            .and_downcast::<Camera>()
            .expect("The item has to be a `Camera`.");

        let label = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .child()
            .and_downcast::<Label>()
            .expect("The child has to be a `Label`.");

        label.set_label(&get_name(&cam));
    });

    let device_selection_dropdown = DropDown::builder()
        .factory(&factory)
        .model(device_provider)
        .show_arrow(true)
        .build();

    device_selection_dropdown.connect_selected_item_notify(move |cb| {
        let selected = cb.selected();
        let dp = DeviceProvider::instance();
        let camera = match dp.camera(selected) {
            Some(c) => c,
            None => { 
                eprintln!("Device provider knows no camera at position {}", selected);
                return;
            }
        };
        let path = get_path(&camera);

        pref_groups
            .as_ref()
            .borrow_mut()
            .switch_device(path);

        caps_panel.borrow_mut().update(&camera);
        camera_view.set_camera(Some(camera));
    });

    device_selection_box.append(&Label::new(Some("Select camera: ")));
    device_selection_box.append(&device_selection_dropdown);

    return device_selection_box;
}

pub fn get_path(camera: &Camera) -> String {
    let props = camera.properties();

    let path_value = match props.get("api.v4l2.path") {
        Some(v) => v,
        None => {
            eprintln!("No device path available as camera-property 'api.v4l2.path'.\nSwitching to default /dev/video0.");
            return "/dev/video0".to_string()
        },
    };
    
    match path_value.get::<String>() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error casting 'api.v4l2.path' to a string: {}\nSwitching to default /dev/video0.", e.to_string());
            return "/dev/video0".to_string()
        }
    }
}

pub fn get_name(camera: &Camera) -> String {
    let props = camera.properties();

    let path_value = match props.get("api.v4l2.cap.card") {
        Some(v) => v,
        None => {
            eprintln!("Property 'api.v4l2.path' not existing on camera.");
            return "Unknown".to_string()
        },
    };
    
    match path_value.get::<String>() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error casting 'api.v4l2.cap.card' to a string: {}", e.to_string());
            return "Unknown".to_string()
        }
    }
}
