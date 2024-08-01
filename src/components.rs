use adw::{prelude::*, ActionRow, PreferencesRow};
use gtk::{Box, Label, Orientation};

/// Creates a preconfigured, horizontal box
pub fn create_hbox() -> Box {
    Box::builder()
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build()
}

/// Creates a pre-configured GtkLabel
pub fn create_label(label: String) -> Label {
    Label::builder()
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .label(label.as_str())
        .tooltip_text(label.as_str())
        .max_width_chars(12)
        .xalign(0.0)
        .build()
}

/// Create a pre-configured row for a preference, that can be added to a
/// PreferencesPage.
///
/// The Box already contains a label, with the given string as text.
/// Append to the box as needed.
pub fn create_pref_row_with_box_and_label(label: String) -> (PreferencesRow, Box) {
    let row = PreferencesRow::new();
    let label = create_label(label);

    let rowbox = create_hbox();
    rowbox.append(&label);

    row.set_child(Some(&rowbox));

    (row, rowbox)
}

/// Create a box to use in preferences, only to present infos.
pub fn create_info_row(label: String, info: String) -> ActionRow {
    ActionRow::builder()
        .title(label)
        .subtitle(info)
        .subtitle_selectable(true)
        .build()
}
