use crate::DomAttribute;
use headless_primitives_core::collapsible::{CollapsibleModel, CollapsibleState};

pub fn collapsible_trigger_attrs(
    model: &CollapsibleModel,
    controls: Option<&str>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let state = collapsible_state_value(model.state());
    attrs.push(DomAttribute::string("data-state", state));
    attrs.push(DomAttribute::string(
        "aria-expanded",
        if model.open() { "true" } else { "false" },
    ));
    attrs.push(DomAttribute::bool("disabled", model.disabled()));
    if model.disabled() {
        attrs.push(DomAttribute::bool("data-disabled", true));
    }
    if let Some(controls) = controls {
        attrs.push(DomAttribute::string("aria-controls", controls));
    }
    attrs
}

pub fn collapsible_content_attrs(
    model: &CollapsibleModel,
    content_id: Option<&str>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let state = collapsible_state_value(model.state());
    attrs.push(DomAttribute::string("data-state", state));
    attrs.push(DomAttribute::bool("hidden", !model.open()));
    if let Some(id) = content_id {
        attrs.push(DomAttribute::string("id", id));
    }
    attrs
}

fn collapsible_state_value(state: CollapsibleState) -> &'static str {
    match state {
        CollapsibleState::Open => "open",
        CollapsibleState::Closed => "closed",
    }
}

#[cfg(test)]
mod tests {
    use super::{collapsible_content_attrs, collapsible_trigger_attrs};
    use crate::DomAttributeValue;
    use headless_primitives_core::collapsible::CollapsibleModel;

    #[test]
    fn trigger_attrs_include_state_and_controls() {
        let model = CollapsibleModel::new(true);
        let attrs = collapsible_trigger_attrs(&model, Some("content"));
        assert!(attrs.iter().any(|attr| attr.name() == "data-state"));
        assert!(attrs.iter().any(|attr| attr.name() == "aria-controls"));
    }

    #[test]
    fn content_attrs_hide_when_closed() {
        let model = CollapsibleModel::new(false);
        let attrs = collapsible_content_attrs(&model, None);
        let hidden = attrs
            .iter()
            .find(|attr| attr.name() == "hidden")
            .expect("hidden attr");
        assert_eq!(hidden.value(), &DomAttributeValue::Bool(true));
    }
}
