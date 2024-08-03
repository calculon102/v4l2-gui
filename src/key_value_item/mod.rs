use gtk::glib::Object;

mod imp;

gtk::glib::wrapper! {
    pub struct KeyValueItem(ObjectSubclass<imp::KeyValueItem>);
}

impl KeyValueItem {
    pub fn new(id: u32, label: &str) -> Self {
        Object::builder()
            .property("id", id)
            .property("label", label)
            .build()
    }
}

#[derive(Default, Clone)]
pub struct ItemData {
    pub id: u32,
    pub label: String,
}

