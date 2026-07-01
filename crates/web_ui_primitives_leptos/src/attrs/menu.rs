//! Attribute helpers for menu state.

use crate::DomAttribute;
use web_ui_primitives_core::menu::{MenuModel, MenuState};

/// Element contract for menu trigger attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTriggerElement {
    Button,
    Generic,
}

/// Element contract for menu item attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItemElement {
    Button,
    Generic,
}

/// Menu item ARIA contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItemKind {
    Item,
    Radio,
}

/// Disabled-state policy for menu button attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItemDisabledPolicy {
    Native,
    Aria,
}

/// Attribute options for a menu trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuTriggerAttrs<'a> {
    controls_id: Option<&'a str>,
    element: MenuTriggerElement,
}

impl Default for MenuTriggerAttrs<'_> {
    fn default() -> Self {
        Self {
            controls_id: None,
            element: MenuTriggerElement::Button,
        }
    }
}

impl<'a> MenuTriggerAttrs<'a> {
    /// Creates menu trigger attribute options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the controlled menu id.
    pub fn controls_id(mut self, controls_id: &'a str) -> Self {
        self.controls_id = Some(controls_id);
        self
    }

    /// Sets the trigger element contract.
    pub fn element(mut self, element: MenuTriggerElement) -> Self {
        self.element = element;
        self
    }
}

/// Attribute options for menu content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MenuContentAttrs<'a> {
    content_id: Option<&'a str>,
    labelled_by: Option<&'a str>,
}

impl<'a> MenuContentAttrs<'a> {
    /// Creates menu content attribute options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the menu content id.
    pub fn content_id(mut self, content_id: &'a str) -> Self {
        self.content_id = Some(content_id);
        self
    }

    /// Sets `aria-labelledby`.
    pub fn labelled_by(mut self, labelled_by: &'a str) -> Self {
        self.labelled_by = Some(labelled_by);
        self
    }
}

/// Attribute options for a menu item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItemAttrs<'a> {
    item_id: Option<&'a str>,
    element: MenuItemElement,
    kind: MenuItemKind,
    disabled_policy: MenuItemDisabledPolicy,
}

impl Default for MenuItemAttrs<'_> {
    fn default() -> Self {
        Self {
            item_id: None,
            element: MenuItemElement::Button,
            kind: MenuItemKind::Item,
            disabled_policy: MenuItemDisabledPolicy::Native,
        }
    }
}

impl<'a> MenuItemAttrs<'a> {
    /// Creates menu item attribute options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the menu item id.
    pub fn item_id(mut self, item_id: &'a str) -> Self {
        self.item_id = Some(item_id);
        self
    }

    /// Sets the item element contract.
    pub fn element(mut self, element: MenuItemElement) -> Self {
        self.element = element;
        self
    }

    /// Sets the item ARIA contract.
    pub fn kind(mut self, kind: MenuItemKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the disabled-state policy.
    pub fn disabled_policy(mut self, disabled_policy: MenuItemDisabledPolicy) -> Self {
        self.disabled_policy = disabled_policy;
        self
    }
}

/// Returns trigger attributes for a menu toggle element.
pub fn menu_trigger_attrs(model: &MenuModel, options: MenuTriggerAttrs<'_>) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let state = menu_state_value(model.state());
    attrs.push(DomAttribute::string("data-state", state));
    attrs.push(DomAttribute::string("aria-haspopup", "menu"));
    attrs.push(DomAttribute::string(
        "aria-expanded",
        if model.open() { "true" } else { "false" },
    ));
    if options.element == MenuTriggerElement::Button {
        attrs.push(DomAttribute::string("type", "button"));
    }
    if let Some(controls) = options.controls_id {
        attrs.push(DomAttribute::string("aria-controls", controls));
    }
    attrs
}

/// Returns content attributes for a menu surface element.
pub fn menu_content_attrs(model: &MenuModel, options: MenuContentAttrs<'_>) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let state = menu_state_value(model.state());
    attrs.push(DomAttribute::string("data-state", state));
    attrs.push(DomAttribute::string("role", "menu"));
    attrs.push(DomAttribute::bool("hidden", !model.open()));
    attrs.push(DomAttribute::string("tabindex", "-1"));
    if let Some(id) = options.content_id {
        attrs.push(DomAttribute::string("id", id));
    }
    if let Some(labelled_by) = options.labelled_by {
        attrs.push(DomAttribute::string("aria-labelledby", labelled_by));
    }
    attrs
}

/// Returns attributes for a single menu item.
pub fn menu_item_attrs(
    model: &MenuModel,
    index: usize,
    options: MenuItemAttrs<'_>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let checked = model.item_checked(index);
    let disabled = model.disabled(index);
    let focused = model.focused() == Some(index);
    attrs.push(DomAttribute::string(
        "role",
        match options.kind {
            MenuItemKind::Item => "menuitem",
            MenuItemKind::Radio => "menuitemradio",
        },
    ));
    if options.element == MenuItemElement::Button {
        attrs.push(DomAttribute::string("type", "button"));
    }
    attrs.push(DomAttribute::string(
        "tabindex",
        if focused && !disabled { "0" } else { "-1" },
    ));
    if focused && !disabled {
        attrs.push(DomAttribute::bool("data-highlighted", true));
    }
    if options.kind == MenuItemKind::Radio {
        attrs.push(DomAttribute::string(
            "data-state",
            if checked { "checked" } else { "unchecked" },
        ));
        attrs.push(DomAttribute::string(
            "aria-checked",
            if checked { "true" } else { "false" },
        ));
    }
    if options.element == MenuItemElement::Button
        && options.disabled_policy == MenuItemDisabledPolicy::Native
    {
        attrs.push(DomAttribute::bool("disabled", disabled));
    }
    if disabled {
        attrs.push(DomAttribute::string("aria-disabled", "true"));
        attrs.push(DomAttribute::bool("data-disabled", true));
    }
    if let Some(id) = options.item_id {
        attrs.push(DomAttribute::string("id", id));
    }
    attrs
}

/// Returns attributes for a menu item checked indicator.
pub fn menu_item_indicator_attrs(model: &MenuModel, index: usize) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let checked = model.item_checked(index);
    attrs.push(DomAttribute::bool("hidden", !checked));
    attrs.push(DomAttribute::string(
        "data-state",
        if checked { "checked" } else { "unchecked" },
    ));
    attrs
}

fn menu_state_value(state: MenuState) -> &'static str {
    match state {
        MenuState::Open => "open",
        MenuState::Closed => "closed",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MenuContentAttrs, MenuItemAttrs, MenuItemDisabledPolicy, MenuItemElement, MenuItemKind,
        MenuTriggerAttrs, MenuTriggerElement, menu_content_attrs, menu_item_attrs,
        menu_item_indicator_attrs, menu_trigger_attrs,
    };
    use crate::DomAttributeValue;
    use web_ui_primitives_core::menu::MenuModel;

    #[test]
    fn trigger_attrs_include_menu_button_contract() {
        let mut model = MenuModel::new(2);
        model.set_open(true);
        let attrs = menu_trigger_attrs(&model, MenuTriggerAttrs::new().controls_id("locale-menu"));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-haspopup"
                && attr.value() == &DomAttributeValue::String("menu".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-expanded"
                && attr.value() == &DomAttributeValue::String("true".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "type"
                && attr.value() == &DomAttributeValue::String("button".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-controls"
                && attr.value() == &DomAttributeValue::String("locale-menu".to_string())
        }));
    }

    #[test]
    fn trigger_attrs_can_target_generic_element() {
        let model = MenuModel::new(1);
        let attrs = menu_trigger_attrs(
            &model,
            MenuTriggerAttrs::new().element(MenuTriggerElement::Generic),
        );
        assert!(!attrs.iter().any(|attr| attr.name() == "type"));
    }

    #[test]
    fn content_attrs_include_menu_contract() {
        let model = MenuModel::new(1);
        let attrs = menu_content_attrs(
            &model,
            MenuContentAttrs::new()
                .content_id("menu")
                .labelled_by("trigger"),
        );
        assert!(attrs.iter().any(|attr| {
            attr.name() == "role" && attr.value() == &DomAttributeValue::String("menu".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "hidden" && attr.value() == &DomAttributeValue::Bool(true)
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "id" && attr.value() == &DomAttributeValue::String("menu".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-labelledby"
                && attr.value() == &DomAttributeValue::String("trigger".to_string())
        }));
    }

    #[test]
    fn item_attrs_include_radio_checked_state() {
        let mut model = MenuModel::with_checked(2, Some(1));
        model.set_open(true);
        let attrs = menu_item_attrs(
            &model,
            1,
            MenuItemAttrs::new()
                .item_id("item")
                .kind(MenuItemKind::Radio),
        );
        assert!(attrs.iter().any(|attr| {
            attr.name() == "role"
                && attr.value() == &DomAttributeValue::String("menuitemradio".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-checked"
                && attr.value() == &DomAttributeValue::String("true".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "data-state"
                && attr.value() == &DomAttributeValue::String("checked".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "id" && attr.value() == &DomAttributeValue::String("item".to_string())
        }));
    }

    #[test]
    fn item_attrs_reflect_disabled_state() {
        let mut model = MenuModel::new(2);
        model.set_disabled(1, true);
        model.set_open(true);
        let attrs = menu_item_attrs(&model, 1, MenuItemAttrs::new());
        assert!(attrs.iter().any(|attr| {
            attr.name() == "disabled" && attr.value() == &DomAttributeValue::Bool(true)
        }));
        assert!(attrs.iter().any(|attr| attr.name() == "aria-disabled"));
        assert!(attrs.iter().any(|attr| attr.name() == "data-disabled"));
    }

    #[test]
    fn item_attrs_can_use_aria_disabled_policy_for_buttons() {
        let mut model = MenuModel::new(2);
        model.set_disabled(1, true);
        let attrs = menu_item_attrs(
            &model,
            1,
            MenuItemAttrs::new().disabled_policy(MenuItemDisabledPolicy::Aria),
        );
        assert!(!attrs.iter().any(|attr| attr.name() == "disabled"));
        assert!(attrs.iter().any(|attr| attr.name() == "aria-disabled"));
    }

    #[test]
    fn item_attrs_can_target_generic_element() {
        let mut model = MenuModel::new(2);
        model.set_disabled(1, true);
        let attrs = menu_item_attrs(
            &model,
            1,
            MenuItemAttrs::new().element(MenuItemElement::Generic),
        );
        assert!(!attrs.iter().any(|attr| attr.name() == "type"));
        assert!(!attrs.iter().any(|attr| attr.name() == "disabled"));
        assert!(attrs.iter().any(|attr| attr.name() == "aria-disabled"));
    }

    #[test]
    fn item_indicator_attrs_hide_when_unchecked() {
        let model = MenuModel::with_checked(2, Some(1));
        let attrs = menu_item_indicator_attrs(&model, 0);
        assert!(attrs.iter().any(|attr| {
            attr.name() == "hidden" && attr.value() == &DomAttributeValue::Bool(true)
        }));
    }
}
