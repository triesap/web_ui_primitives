use crate::PrimitiveAttribute;
use ui_primitives_core::dialog::{DialogModel, DialogState};

pub fn dialog_trigger_attrs(model: &DialogModel, controls_id: Option<&str>) -> Vec<PrimitiveAttribute> {
    let mut attrs = Vec::new();
    let state = dialog_state_value(model.state());
    attrs.push(PrimitiveAttribute::string("data-state", state));
    attrs.push(PrimitiveAttribute::string("aria-haspopup", "dialog"));
    attrs.push(PrimitiveAttribute::string(
        "aria-expanded",
        if model.open() { "true" } else { "false" },
    ));
    if let Some(controls) = controls_id {
        attrs.push(PrimitiveAttribute::string("aria-controls", controls));
    }
    attrs
}

pub fn dialog_content_attrs(
    model: &DialogModel,
    labelled_by: Option<&str>,
    described_by: Option<&str>,
) -> Vec<PrimitiveAttribute> {
    let mut attrs = Vec::new();
    let state = dialog_state_value(model.state());
    attrs.push(PrimitiveAttribute::string("data-state", state));
    attrs.push(PrimitiveAttribute::string("role", "dialog"));
    attrs.push(PrimitiveAttribute::string("tabindex", "-1"));
    if model.modal() {
        attrs.push(PrimitiveAttribute::string("aria-modal", "true"));
    }
    if let Some(label) = labelled_by {
        attrs.push(PrimitiveAttribute::string("aria-labelledby", label));
    }
    if let Some(description) = described_by {
        attrs.push(PrimitiveAttribute::string("aria-describedby", description));
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
    use crate::PrimitiveAttributeValue;
    use ui_primitives_core::dialog::DialogModel;

    #[test]
    fn trigger_attrs_reflect_open() {
        let model = DialogModel::new(true);
        let attrs = dialog_trigger_attrs(&model, Some("dialog"));
        let expanded = attrs
            .iter()
            .find(|attr| attr.name() == "aria-expanded")
            .expect("aria-expanded");
        assert_eq!(
            expanded.value(),
            &PrimitiveAttributeValue::String("true".to_string())
        );
    }

    #[test]
    fn content_attrs_include_role_and_modal() {
        let model = DialogModel::new(true);
        let attrs = dialog_content_attrs(&model, None, None);
        assert!(attrs.iter().any(|attr| attr.name() == "role"));
        assert!(attrs.iter().any(|attr| attr.name() == "aria-modal"));
    }
}
