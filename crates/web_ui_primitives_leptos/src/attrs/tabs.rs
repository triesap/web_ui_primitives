//! Attribute helpers for tabs state.

use crate::DomAttribute;
use web_ui_primitives_core::orientation::Orientation;
use web_ui_primitives_core::tabs::TabsModel;

/// Element contract for tab trigger attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsTriggerElement {
    Button,
    Generic,
}

/// Disabled-state policy for tab trigger attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsTriggerDisabledPolicy {
    Native,
    Aria,
}

/// Attribute options for a tab trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabsTriggerAttrs<'a> {
    trigger_id: Option<&'a str>,
    controls_id: Option<&'a str>,
    element: TabsTriggerElement,
    disabled_policy: TabsTriggerDisabledPolicy,
}

impl Default for TabsTriggerAttrs<'_> {
    fn default() -> Self {
        Self {
            trigger_id: None,
            controls_id: None,
            element: TabsTriggerElement::Button,
            disabled_policy: TabsTriggerDisabledPolicy::Native,
        }
    }
}

impl<'a> TabsTriggerAttrs<'a> {
    /// Creates tab trigger attribute options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tab trigger `id`.
    pub fn trigger_id(mut self, trigger_id: &'a str) -> Self {
        self.trigger_id = Some(trigger_id);
        self
    }

    /// Sets the controlled panel id.
    pub fn controls_id(mut self, controls_id: &'a str) -> Self {
        self.controls_id = Some(controls_id);
        self
    }

    /// Sets the trigger element contract.
    pub fn element(mut self, element: TabsTriggerElement) -> Self {
        self.element = element;
        self
    }

    /// Sets the disabled-state policy.
    pub fn disabled_policy(mut self, disabled_policy: TabsTriggerDisabledPolicy) -> Self {
        self.disabled_policy = disabled_policy;
        self
    }
}

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
    options: TabsTriggerAttrs<'_>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let selected = model.selected() == Some(index);
    let focused = model.focused() == Some(index);
    let disabled = model.disabled(index);
    attrs.push(DomAttribute::string("role", "tab"));
    if options.element == TabsTriggerElement::Button {
        attrs.push(DomAttribute::string("type", "button"));
    }
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
    if options.element == TabsTriggerElement::Button
        && options.disabled_policy == TabsTriggerDisabledPolicy::Native
    {
        attrs.push(DomAttribute::bool("disabled", disabled));
    }
    if disabled {
        attrs.push(DomAttribute::string("aria-disabled", "true"));
        attrs.push(DomAttribute::bool("data-disabled", true));
    }
    if let Some(id) = options.trigger_id {
        attrs.push(DomAttribute::string("id", id));
    }
    if let Some(controls) = options.controls_id {
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
    use super::{
        TabsTriggerAttrs, TabsTriggerDisabledPolicy, TabsTriggerElement, tabs_list_attrs,
        tabs_panel_attrs, tabs_trigger_attrs,
    };
    use crate::DomAttributeValue;
    use web_ui_primitives_core::orientation::Orientation;
    use web_ui_primitives_core::tabs::TabsModel;

    #[test]
    fn list_attrs_include_orientation_for_vertical() {
        let attrs = tabs_list_attrs(Orientation::Vertical);
        assert!(attrs.iter().any(|attr| attr.name() == "aria-orientation"));
    }

    #[test]
    fn trigger_attrs_selected_state() {
        let model = TabsModel::new(2);
        let attrs = tabs_trigger_attrs(&model, 0, TabsTriggerAttrs::new());
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
        let attrs = tabs_trigger_attrs(&model, 1, TabsTriggerAttrs::new());
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
    fn trigger_attrs_include_native_button_contract_by_default() {
        let model = TabsModel::new(2);
        let attrs = tabs_trigger_attrs(
            &model,
            0,
            TabsTriggerAttrs::new()
                .trigger_id("tab-a")
                .controls_id("panel-a"),
        );
        assert!(attrs.iter().any(|attr| {
            attr.name() == "type"
                && attr.value() == &DomAttributeValue::String("button".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "id" && attr.value() == &DomAttributeValue::String("tab-a".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-controls"
                && attr.value() == &DomAttributeValue::String("panel-a".to_string())
        }));
    }

    #[test]
    fn trigger_attrs_can_use_aria_disabled_policy_for_buttons() {
        let mut model = TabsModel::new(2);
        model.set_disabled(1, true);
        let attrs = tabs_trigger_attrs(
            &model,
            1,
            TabsTriggerAttrs::new().disabled_policy(TabsTriggerDisabledPolicy::Aria),
        );
        assert!(!attrs.iter().any(|attr| attr.name() == "disabled"));
        assert!(attrs.iter().any(|attr| attr.name() == "aria-disabled"));
    }

    #[test]
    fn trigger_attrs_can_target_generic_elements() {
        let mut model = TabsModel::new(2);
        model.set_disabled(1, true);
        let attrs = tabs_trigger_attrs(
            &model,
            1,
            TabsTriggerAttrs::new().element(TabsTriggerElement::Generic),
        );
        assert!(!attrs.iter().any(|attr| attr.name() == "type"));
        assert!(!attrs.iter().any(|attr| attr.name() == "disabled"));
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
