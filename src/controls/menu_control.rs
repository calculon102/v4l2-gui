use std::rc::Rc;

use adw::ComboRow;
use adw::PreferencesRow;
use adw::prelude::*;
use glib::subclass::shared::RefCounted;
use gtk::gio::ListStore;
use gtk::Label;
use gtk::ListItem;
use gtk::SignalListItemFactory;
use v4l::{control::Description, Device};

use crate::key_value_item::KeyValueItem;

use super::{ControlUi, ControlValueError};

pub struct MenuControl {
    device: Rc<Device>,
    combo_row: Rc<ComboRow>,
}

impl MenuControl {
    pub fn new(device: Rc<Device>, description: &Description, on_change: Rc<Box<dyn Fn() + 'static>>) -> Self {
        let readonly = description.flags.contains(v4l::control::Flags::READ_ONLY);
        let inactive = description.flags.contains(v4l::control::Flags::INACTIVE);

        let value = Self::query_state(device.as_ref(), &description);

        let ctrl_items = match &description.items {
            Some(i) => i,
            None => panic!("No menu items found for {}", description.name),
        };

        let store = ListStore::with_type(KeyValueItem::static_type());
        let mut selected_position = 0;
        let mut count = 0;
        for item in ctrl_items {
            store.append(&KeyValueItem::new(item.0, &item.1.to_string()));
            if value as u32 == item.0 {
                selected_position = count;
                break;
            }
            count += 1;
        }

        let factory = SignalListItemFactory::new();
        factory.connect_setup(|_, list_item| {
            let label = Label::new(None);
            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&label));
        });

        factory.connect_bind(|_, list_item| {
            let key_value_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .item()
                .and_downcast::<KeyValueItem>()
                .expect("The item has to be an `KeyValueItem`.");

            // Get `Label` from `ListItem`
            let label = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<Label>()
                .expect("The child has to be a `Label`.");

            // Set "label" to "number"
            label.set_label(&key_value_item.label().to_string());
        });

        let row = ComboRow::builder()
            .factory(&factory)
            .model(&store)
            .sensitive(!readonly && !inactive)
            .title(description.name.clone())
            .build();

        // TODO What if the given value was not selectable?
        row.set_selected(selected_position);

        let id_copy = description.id;
        let dev_copy = device.clone();
        row.connect_selected_item_notify(move |row| {
            let item = row
                .model()
                .expect("There has to be a model.")
                .item(row.selected())
                .and_downcast::<KeyValueItem>()
                .expect("The item has to be a `KeyValueItem`.");

            let new_value = v4l::control::Value::Integer(item.id() as i64);
            let new_control = v4l::control::Control {
                id: id_copy,
                value: new_value,
            };

            match dev_copy.set_control(new_control) {
                Ok(_) => on_change(), 
                Err(e) => eprintln!("Error setting control: {}", e),
            };
        });

        MenuControl {
            device: device.clone(),
            combo_row: Rc::new(row),
        }
    } 

    // TODO Duplicate in IntegerControl
    fn query_control_integer(
        device: &Device,
        ctrl_desc: &Description,
    ) -> Result<i64, ControlValueError> {
        let control = match device.control(ctrl_desc.id) {
            Ok(v) => v,
            Err(e) => return Err(ControlValueError::new(e.to_string())),
        };

        return match control.value {
            v4l::control::Value::Integer(int_val) => Ok(int_val),
            _ => Err(ControlValueError::new(format!(
                "Value of {} is not an integer",
                ctrl_desc.name
            ))),
        };
    }

    // TODO Duplicate in IntegerControl
    fn query_state(device: &Device, description: &Description) -> i64 {
        match Self::query_control_integer(&device, &description) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error while checking state of control: {}", e.message);
                description.default
            }
        }
    }
}

impl ControlUi for MenuControl {
    fn preference_row(&self) -> Rc<PreferencesRow> {
        // TODO Safe method
        let row = unsafe {
            let ptr = self.combo_row.clone().into_raw();
            Rc::<PreferencesRow>::from_raw(ptr.cast())
        };
        row
    }

    fn update_value(&self, description: &Description) {
        let value = MenuControl::query_state(self.device.as_ref(), description);
        
        let ctrl_items = match &description.items {
            Some(i) => i,
            None => panic!("No menu items found for {}", description.name),
        };

        let mut new_selection = 0;
        let mut count = 0;
        for item in ctrl_items {
            if value as u32 == item.0 {
                new_selection = count;
                break;
            }
            count += 1;
        }
        
        let old_selection = self.combo_row.selected();
        if new_selection != old_selection {
            self.combo_row.set_selected(new_selection);
        }
    }

    fn update_state(&self, description: &Description) {
        let readonly = description.flags.contains(v4l::control::Flags::READ_ONLY);
        let inactive = description.flags.contains(v4l::control::Flags::INACTIVE);

        self.combo_row.set_sensitive(!readonly && !inactive);
    }
}

