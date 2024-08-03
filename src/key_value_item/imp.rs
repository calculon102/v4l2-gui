use std::cell::RefCell;

use gtk::subclass::prelude::*;
use glib::Properties;
use gtk::prelude::*;

use super::ItemData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::KeyValueItem)]
pub struct KeyValueItem {
    #[property(name = "id", get, set, type = u32, member = id)]
    #[property(name = "label", get, set, type = String, member = label)]
    pub data: RefCell<ItemData>,
}

#[glib::object_subclass]
impl ObjectSubclass for KeyValueItem {
    const NAME: &'static str = "KeyValueItem";
    type Type = super::KeyValueItem;
}

#[glib::derived_properties]
impl ObjectImpl for KeyValueItem {}

