use super::ERR_COLOR;
use egui::{epaint::text::LayoutJob, FontId, TextFormat, Ui};
use pretty_type_name::pretty_type_name_str;

pub fn unconstructable_variant(
    ui: &mut Ui,
    type_name: &str,
    variant: &str,
    unconstructable_field_types: &[&str],
) {
    let pretty_name = pretty_type_name_str(type_name);
    let qualified_variant = format!("{}::{}", pretty_name, variant);

    let mut vec = Vec::with_capacity(2 + unconstructable_field_types.len() * 2 + 4);

    vec.extend([
        (FontId::monospace(12.0), qualified_variant.as_str()),
        (
            FontId::proportional(13.0),
            " has unconstructable fields.\nConsider adding ",
        ),
        (FontId::monospace(12.0), "#[reflect(Default)]"),
        (FontId::proportional(13.0), " to\n\n"),
    ]);
    vec.extend(unconstructable_field_types.iter().flat_map(|variant| {
        [
            (FontId::proportional(13.0), "- "),
            (FontId::monospace(12.0), *variant),
        ]
    }));

    ui.label(
        vec.drain(..)
            .fold(LayoutJob::default(), |mut job, (font_id, text)| {
                job.append(text, 0.0, TextFormat::simple(font_id, ERR_COLOR));
                job
            }),
    );
}

pub fn reflect_value_no_impl(ui: &mut egui::Ui, type_name: &str) -> egui::Response {
    let pretty_name = pretty_type_name::pretty_type_name_str(type_name);

    let text_data = [
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " is "),
        (FontId::monospace(12.0), "#[reflect_value]"),
        (FontId::proportional(13.0), ", but has no "),
        (FontId::monospace(12.0), "InspectableImpl"),
        (FontId::proportional(13.0), " registered in the "),
        (FontId::monospace(12.0), "TypeRegistry"),
        (FontId::proportional(13.0), " .\n"),
        (FontId::proportional(13.0), "Try calling "),
        (
            FontId::monospace(12.0),
            &format!(".register_type::<{}>", pretty_name),
        ),
        (FontId::proportional(13.0), " or add the "),
        (FontId::monospace(12.0), "InspectorPlugin"),
        (FontId::proportional(13.0), " for builtin types."),
    ];

    ui.label(text_data.into_iter().fold(
        egui::epaint::text::LayoutJob::default(),
        |mut job, (font_id, text)| {
            job.append(text, 0.0, egui::TextFormat::simple(font_id, ERR_COLOR));
            job
        },
    ))
}

pub fn no_default_value(ui: &mut Ui, type_name: &str) {
    let text = [
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " has no "),
        (FontId::monospace(12.0), "ReflectDefault"),
        (
            FontId::proportional(13.0),
            " type data, so no value of it can be constructed.",
        ),
    ];

    ui.label(text.into_iter().fold(
        egui::epaint::text::LayoutJob::default(),
        |mut job, (font_id, text)| {
            job.append(text, 0.0, egui::TextFormat::simple(font_id, ERR_COLOR));
            job
        },
    ));
}
