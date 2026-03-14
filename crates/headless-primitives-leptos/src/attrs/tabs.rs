//! Attribute helpers for tabs state.

use crate::DomAttribute;
use headless_primitives_core::orientation::Orientation;
use headless_primitives_core::tabs::TabsModel;

/// Returns container attributes for a tabs list element.
pub fn tabs_list_attrs(orientation: Orientation) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    attrs.push(DomAttribute::string("role", "tablist"));
    if orientation == Orientation::Vertical {
        attrs.push(DomAttribute::string(
            "aria-orientation",
            orientation.as_aria_value(),
        ));
    }
    attrs
}

/// Returns trigger attributes for a single tab button.
///
/// Disabled state is derived from `model`, so callers should not layer a
/// separate disabled flag on top of these attributes.
pub fn tabs_trigger_attrs(
    model: &TabsModel,
    index: usize,
    trigger_id: Option<&str>,
    controls_id: Option<&str>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let selected = model.selected() == Some(index);
    let focused = model.focused() == Some(index);
    let disabled = model.disabled(index);
    attrs.push(DomAttribute::string("role", "tab"));
    attrs.push(DomAttribute::string(
        "aria-selected",
        if selected { "true" } else { "false" },
    ));
    attrs.push(DomAttribute::string(
        "data-state",
        if selected { "active" } else { "inactive" },
    ));
    attrs.push(DomAttribute::string(
        "tabindex",
        if focused && !disabled { "0" } else { "-1" },
    ));
    attrs.push(DomAttribute::bool("disabled", disabled));
    if disabled {
        attrs.push(DomAttribute::string("aria-disabled", "true"));
        attrs.push(DomAttribute::bool("data-disabled", true));
    }
    if let Some(id) = trigger_id {
        attrs.push(DomAttribute::string("id", id));
    }
    if let Some(controls) = controls_id {
        attrs.push(DomAttribute::string("aria-controls", controls));
    }
    attrs
}

/// Returns panel attributes for a single tab panel.
pub fn tabs_panel_attrs(
    model: &TabsModel,
    index: usize,
    panel_id: Option<&str>,
    labelled_by: Option<&str>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let selected = model.selected() == Some(index);
    attrs.push(DomAttribute::string("role", "tabpanel"));
    attrs.push(DomAttribute::bool("hidden", !selected));
    attrs.push(DomAttribute::string("tabindex", "0"));
    if let Some(id) = panel_id {
        attrs.push(DomAttribute::string("id", id));
    }
    if let Some(labelled_by) = labelled_by {
        attrs.push(DomAttribute::string("aria-labelledby", labelled_by));
    }
    attrs
}

#[cfg(test)]
mod tests {
    use super::{tabs_list_attrs, tabs_panel_attrs, tabs_trigger_attrs};
    use crate::DomAttributeValue;
    use headless_primitives_core::orientation::Orientation;
    use headless_primitives_core::tabs::TabsModel;

    #[test]
    fn list_attrs_include_orientation_for_vertical() {
        let attrs = tabs_list_attrs(Orientation::Vertical);
        assert!(attrs.iter().any(|attr| attr.name() == "aria-orientation"));
    }

    #[test]
    fn trigger_attrs_selected_state() {
        let model = TabsModel::new(2);
        let attrs = tabs_trigger_attrs(&model, 0, None, None);
        let state = attrs
            .iter()
            .find(|attr| attr.name() == "data-state")
            .expect("data-state");
        assert_eq!(
            state.value(),
            &DomAttributeValue::String("active".to_string())
        );
    }

    #[test]
    fn trigger_attrs_reflect_model_disabled_state() {
        let mut model = TabsModel::new(2);
        model.set_disabled(1, true);
        let attrs = tabs_trigger_attrs(&model, 1, None, None);
        let disabled = attrs
            .iter()
            .find(|attr| attr.name() == "disabled")
            .expect("disabled");
        let tabindex = attrs
            .iter()
            .find(|attr| attr.name() == "tabindex")
            .expect("tabindex");
        assert_eq!(disabled.value(), &DomAttributeValue::Bool(true));
        assert_eq!(
            tabindex.value(),
            &DomAttributeValue::String("-1".to_string())
        );
        assert!(attrs.iter().any(|attr| attr.name() == "aria-disabled"));
    }

    #[test]
    fn panel_attrs_hidden_when_unselected() {
        let model = TabsModel::new(2);
        let attrs = tabs_panel_attrs(&model, 1, None, None);
        let hidden = attrs
            .iter()
            .find(|attr| attr.name() == "hidden")
            .expect("hidden");
        assert_eq!(hidden.value(), &DomAttributeValue::Bool(true));
    }
}
