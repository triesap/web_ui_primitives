//! Attribute helpers for dialog state.

use crate::DomAttribute;
use web_ui_primitives_core::dialog::{DialogModel, DialogState};

/// ARIA role for a dialog surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogRole {
    Dialog,
    AlertDialog,
}

impl DialogRole {
    /// Returns the role attribute value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dialog => "dialog",
            Self::AlertDialog => "alertdialog",
        }
    }
}

/// Error returned when a dialog accessible-name value is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogNameError {
    EmptyLabel,
    EmptyLabelledBy,
    EmptyDescribedBy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogNameKind {
    Label,
    LabelledBy,
}

/// Required accessible name for a dialog surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogName<'a> {
    kind: DialogNameKind,
    value: &'a str,
}

impl<'a> DialogName<'a> {
    /// Creates an `aria-label` accessible name.
    pub fn label(value: &'a str) -> Result<Self, DialogNameError> {
        if value.trim().is_empty() {
            return Err(DialogNameError::EmptyLabel);
        }
        Ok(Self {
            kind: DialogNameKind::Label,
            value,
        })
    }

    /// Creates an `aria-labelledby` accessible name.
    pub fn labelled_by(value: &'a str) -> Result<Self, DialogNameError> {
        if value.trim().is_empty() {
            return Err(DialogNameError::EmptyLabelledBy);
        }
        Ok(Self {
            kind: DialogNameKind::LabelledBy,
            value,
        })
    }
}

/// Attribute options for a dialog surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogContentAttrs<'a> {
    role: DialogRole,
    name: DialogName<'a>,
    described_by: Option<&'a str>,
}

impl<'a> DialogContentAttrs<'a> {
    /// Creates dialog content attributes with a required accessible name.
    pub fn new(name: DialogName<'a>) -> Self {
        Self {
            role: DialogRole::Dialog,
            name,
            described_by: None,
        }
    }

    /// Sets the ARIA role for the dialog surface.
    pub fn role(mut self, role: DialogRole) -> Self {
        self.role = role;
        self
    }

    /// Sets `aria-describedby` for the dialog surface.
    pub fn described_by(mut self, value: &'a str) -> Result<Self, DialogNameError> {
        if value.trim().is_empty() {
            return Err(DialogNameError::EmptyDescribedBy);
        }
        self.described_by = Some(value);
        Ok(self)
    }
}

/// Returns trigger attributes for a dialog toggle element.
///
/// The optional `controls_id` is written to `aria-controls` when provided.
pub fn dialog_trigger_attrs(model: &DialogModel, controls_id: Option<&str>) -> Vec<DomAttribute> {
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

/// Returns content attributes for a dialog surface element.
pub fn dialog_content_attrs(
    model: &DialogModel,
    options: DialogContentAttrs<'_>,
) -> Vec<DomAttribute> {
    let mut attrs = Vec::new();
    let state = dialog_state_value(model.state());
    attrs.push(DomAttribute::string("data-state", state));
    attrs.push(DomAttribute::string("role", options.role.as_str()));
    attrs.push(DomAttribute::string("tabindex", "-1"));
    if model.modal() {
        attrs.push(DomAttribute::string("aria-modal", "true"));
    }
    match options.name.kind {
        DialogNameKind::Label => {
            attrs.push(DomAttribute::string("aria-label", options.name.value));
        }
        DialogNameKind::LabelledBy => {
            attrs.push(DomAttribute::string("aria-labelledby", options.name.value));
        }
    }
    if let Some(description) = options.described_by {
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
    use super::{
        DialogContentAttrs, DialogName, DialogNameError, DialogRole, dialog_content_attrs,
        dialog_trigger_attrs,
    };
    use crate::DomAttributeValue;
    use web_ui_primitives_core::dialog::DialogModel;

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
            &DomAttributeValue::String("true".to_string())
        );
    }

    #[test]
    fn content_attrs_include_role_and_modal() {
        let model = DialogModel::new(true);
        let attrs = dialog_content_attrs(
            &model,
            DialogContentAttrs::new(DialogName::labelled_by("dialog-title").expect("name")),
        );
        assert!(attrs.iter().any(|attr| {
            attr.name() == "role"
                && attr.value() == &DomAttributeValue::String("dialog".to_string())
        }));
        assert!(attrs.iter().any(|attr| attr.name() == "aria-modal"));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-labelledby"
                && attr.value() == &DomAttributeValue::String("dialog-title".to_string())
        }));
    }

    #[test]
    fn content_attrs_support_alertdialog_label_and_description() {
        let model = DialogModel::new(true);
        let attrs = dialog_content_attrs(
            &model,
            DialogContentAttrs::new(DialogName::label("Delete item").expect("name"))
                .role(DialogRole::AlertDialog)
                .described_by("dialog-description")
                .expect("description"),
        );
        assert!(attrs.iter().any(|attr| {
            attr.name() == "role"
                && attr.value() == &DomAttributeValue::String("alertdialog".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-label"
                && attr.value() == &DomAttributeValue::String("Delete item".to_string())
        }));
        assert!(attrs.iter().any(|attr| {
            attr.name() == "aria-describedby"
                && attr.value() == &DomAttributeValue::String("dialog-description".to_string())
        }));
    }

    #[test]
    fn content_name_rejects_empty_values() {
        assert_eq!(
            DialogName::label(" ").expect_err("empty label"),
            DialogNameError::EmptyLabel
        );
        assert_eq!(
            DialogName::labelled_by("").expect_err("empty labelledby"),
            DialogNameError::EmptyLabelledBy
        );
        assert_eq!(
            DialogContentAttrs::new(DialogName::label("Dialog").expect("name"))
                .described_by(" ")
                .expect_err("empty describedby"),
            DialogNameError::EmptyDescribedBy
        );
    }
}
