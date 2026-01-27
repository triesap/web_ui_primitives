use crate::PrimitiveAttribute;
use ui_primitives_core::orientation::Orientation;
use ui_primitives_core::tabs::TabsModel;

pub fn tabs_list_attrs(orientation: Orientation) -> Vec<PrimitiveAttribute> {
    let mut attrs = Vec::new();
    attrs.push(PrimitiveAttribute::string("role", "tablist"));
    if orientation == Orientation::Vertical {
        attrs.push(PrimitiveAttribute::string(
            "aria-orientation",
            orientation.as_aria_value(),
        ));
    }
    attrs
}

pub fn tabs_trigger_attrs(
    model: &TabsModel,
    index: usize,
    trigger_id: Option<&str>,
    controls_id: Option<&str>,
    disabled: bool,
) -> Vec<PrimitiveAttribute> {
    let mut attrs = Vec::new();
    let selected = model.selected() == Some(index);
    let focused = model.focused() == Some(index);
    attrs.push(PrimitiveAttribute::string("role", "tab"));
    attrs.push(PrimitiveAttribute::string(
        "aria-selected",
        if selected { "true" } else { "false" },
    ));
    attrs.push(PrimitiveAttribute::string(
        "data-state",
        if selected { "active" } else { "inactive" },
    ));
    attrs.push(PrimitiveAttribute::string(
        "tabindex",
        if focused && !disabled { "0" } else { "-1" },
    ));
    attrs.push(PrimitiveAttribute::bool("disabled", disabled));
    if disabled {
        attrs.push(PrimitiveAttribute::string("aria-disabled", "true"));
    }
    if let Some(id) = trigger_id {
        attrs.push(PrimitiveAttribute::string("id", id));
    }
    if let Some(controls) = controls_id {
        attrs.push(PrimitiveAttribute::string("aria-controls", controls));
    }
    attrs
}

pub fn tabs_panel_attrs(
    model: &TabsModel,
    index: usize,
    panel_id: Option<&str>,
    labelled_by: Option<&str>,
) -> Vec<PrimitiveAttribute> {
    let mut attrs = Vec::new();
    let selected = model.selected() == Some(index);
    attrs.push(PrimitiveAttribute::string("role", "tabpanel"));
    attrs.push(PrimitiveAttribute::bool("hidden", !selected));
    attrs.push(PrimitiveAttribute::string("tabindex", "0"));
    if let Some(id) = panel_id {
        attrs.push(PrimitiveAttribute::string("id", id));
    }
    if let Some(labelled_by) = labelled_by {
        attrs.push(PrimitiveAttribute::string("aria-labelledby", labelled_by));
    }
    attrs
}

#[cfg(test)]
mod tests {
    use super::{tabs_list_attrs, tabs_panel_attrs, tabs_trigger_attrs};
    use crate::PrimitiveAttributeValue;
    use ui_primitives_core::orientation::Orientation;
    use ui_primitives_core::tabs::TabsModel;

    #[test]
    fn list_attrs_include_orientation_for_vertical() {
        let attrs = tabs_list_attrs(Orientation::Vertical);
        assert!(attrs.iter().any(|attr| attr.name() == "aria-orientation"));
    }

    #[test]
    fn trigger_attrs_selected_state() {
        let model = TabsModel::new(2);
        let attrs = tabs_trigger_attrs(&model, 0, None, None, false);
        let state = attrs
            .iter()
            .find(|attr| attr.name() == "data-state")
            .expect("data-state");
        assert_eq!(
            state.value(),
            &PrimitiveAttributeValue::String("active".to_string())
        );
    }

    #[test]
    fn panel_attrs_hidden_when_unselected() {
        let model = TabsModel::new(2);
        let attrs = tabs_panel_attrs(&model, 1, None, None);
        let hidden = attrs
            .iter()
            .find(|attr| attr.name() == "hidden")
            .expect("hidden");
        assert_eq!(hidden.value(), &PrimitiveAttributeValue::Bool(true));
    }
}
