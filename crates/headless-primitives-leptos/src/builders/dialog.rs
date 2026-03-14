use crate::DomAttribute;
use headless_primitives_core::dialog::{DialogModel, DialogState};

pub fn dialog_trigger_attrs(
    model: &DialogModel,
    controls_id: Option<&str>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let state = dialog_state_value(model.state());
    attrs.push(DomAttribute::string("data-state", state));
    attrs.push(DomAttribute::string("aria-haspopup", "dialog"));
    attrs.push(DomAttribute::string(
        "aria-expanded",
        if model.open() { "true" } else { "false" },
    ));
    if let Some(controls) = controls_id {
        attrs.push(DomAttribute::string("aria-controls", controls));
    }
    attrs
}

pub fn dialog_content_attrs(
    model: &DialogModel,
    labelled_by: Option<&str>,
    described_by: Option<&str>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let state = dialog_state_value(model.state());
    attrs.push(DomAttribute::string("data-state", state));
    attrs.push(DomAttribute::string("role", "dialog"));
    attrs.push(DomAttribute::string("tabindex", "-1"));
    if model.modal() {
        attrs.push(DomAttribute::string("aria-modal", "true"));
    }
    if let Some(label) = labelled_by {
        attrs.push(DomAttribute::string("aria-labelledby", label));
    }
    if let Some(description) = described_by {
        attrs.push(DomAttribute::string("aria-describedby", description));
    }
    attrs
}

fn dialog_state_value(state: DialogState) -> &'static str {
    match state {
        DialogState::Open => "open",
        DialogState::Closed => "closed",
    }
}

#[cfg(test)]
mod tests {
    use super::{dialog_content_attrs, dialog_trigger_attrs};
    use crate::DomAttributeValue;
    use headless_primitives_core::dialog::DialogModel;

    #[test]
    fn trigger_attrs_reflect_open() {
        let model = DialogModel::new(true);
        let attrs = dialog_trigger_attrs(&model, Some("dialog"));
        let expanded = attrs
            .iter()
            .find(|attr| attr.name() == "aria-expanded")
            .expect("aria-expanded");
        assert_eq!(expanded.value(), &DomAttributeValue::String("true".to_string()));
    }

    #[test]
    fn content_attrs_include_role_and_modal() {
        let model = DialogModel::new(true);
        let attrs = dialog_content_attrs(&model, None, None);
        assert!(attrs.iter().any(|attr| attr.name() == "role"));
        assert!(attrs.iter().any(|attr| attr.name() == "aria-modal"));
    }
}
