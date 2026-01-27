use leptos::html;
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimitiveAttribute {
    name: String,
    value: PrimitiveAttributeValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveAttributeValue {
    String(String),
    Bool(bool),
}

impl PrimitiveAttribute {
    pub fn string(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: PrimitiveAttributeValue::String(value.into()),
        }
    }

    pub fn bool(name: impl Into<String>, value: bool) -> Self {
        Self {
            name: name.into(),
            value: PrimitiveAttributeValue::Bool(value),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &PrimitiveAttributeValue {
        &self.value
    }
}

#[cfg(target_arch = "wasm32")]
pub type PrimitiveEventHandler = Callback<web_sys::Event>;
#[cfg(not(target_arch = "wasm32"))]
pub type PrimitiveEventHandler = Callback<()>;

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Clone)]
pub struct PrimitiveEvent {
    name: &'static str,
    handler: PrimitiveEventHandler,
}

impl PrimitiveEvent {
    pub fn new(name: &'static str, handler: PrimitiveEventHandler) -> Self {
        Self { name, handler }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[cfg(target_arch = "wasm32")]
pub type PrimitiveTarget = web_sys::Element;
#[cfg(not(target_arch = "wasm32"))]
pub type PrimitiveTarget = ();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveError {
    AttributeUnavailable,
}

pub type PrimitiveResult<T> = Result<T, PrimitiveError>;

#[derive(Clone)]
pub struct PrimitiveElement<E>
where
    E: html::ElementType,
{
    node_ref: NodeRef<E>,
}

impl<E> PrimitiveElement<E>
where
    E: html::ElementType,
{
    pub fn node_ref(&self) -> NodeRef<E> {
        self.node_ref
    }
}

pub fn use_primitive<E>(
    attrs: impl Into<Signal<Vec<PrimitiveAttribute>>>,
    events: Vec<PrimitiveEvent>,
) -> PrimitiveElement<E>
where
    E: html::ElementType,
    E::Output: 'static,
{
    let attrs = attrs.into();
    let node_ref = NodeRef::<E>::new();

    #[cfg(target_arch = "wasm32")]
    {
        attach_primitive(node_ref, attrs.clone(), events.clone());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = attrs;
        let _ = events;
    }

    PrimitiveElement { node_ref }
}

#[cfg(target_arch = "wasm32")]
fn attach_primitive<E>(
    node_ref: NodeRef<E>,
    attrs: Signal<Vec<PrimitiveAttribute>>,
    events: Vec<PrimitiveEvent>,
) where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let prev_attrs: Rc<RefCell<Vec<PrimitiveAttribute>>> = Rc::new(RefCell::new(Vec::new()));
    let prev_attrs_handle = Rc::clone(&prev_attrs);

    node_ref.on_load(move |root| {
        let Ok(element) = root.dyn_into::<web_sys::Element>() else {
            return;
        };

        let element_for_effect = element.clone();
        let prev_attrs = Rc::clone(&prev_attrs_handle);
        let attrs = attrs.clone();
        Effect::new(move || {
            let next = attrs.get();
            let mut previous = prev_attrs.borrow_mut();
            if let Ok(normalized) = apply_attribute_delta(&element_for_effect, &previous, &next) {
                *previous = normalized;
            }
        });

        let mut handles: Vec<EventHandle> = Vec::new();
        for event in events.into_iter() {
            let name = event.name;
            let handler = event.handler;
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                handler.run(event);
            }) as Box<dyn FnMut(_)>);
            let _ = element.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref());
            handles.push(EventHandle { name, closure });
        }

        if !handles.is_empty() {
            let element = element.clone();
            on_cleanup(move || {
                for handle in handles {
                    let _ = element.remove_event_listener_with_callback(
                        handle.name,
                        handle.closure.as_ref().unchecked_ref(),
                    );
                }
            });
        }
    });
}

#[cfg(target_arch = "wasm32")]
struct EventHandle {
    name: &'static str,
    closure: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
}

pub fn apply_attribute_delta(
    target: &PrimitiveTarget,
    previous: &[PrimitiveAttribute],
    next: &[PrimitiveAttribute],
) -> PrimitiveResult<Vec<PrimitiveAttribute>> {
    let AttributeDelta { remove, set } = attribute_delta(previous, next);

    #[cfg(target_arch = "wasm32")]
    {
        for name in remove {
            let _ = target.remove_attribute(&name);
        }

        for attr in &set {
            apply_attribute(target, attr)?;
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = target;
        let _ = remove;
    }

    Ok(set)
}

#[cfg(target_arch = "wasm32")]
fn apply_attribute(target: &PrimitiveTarget, attr: &PrimitiveAttribute) -> PrimitiveResult<()> {
    match attr.value() {
        PrimitiveAttributeValue::String(value) => target
            .set_attribute(attr.name(), value)
            .map_err(|_| PrimitiveError::AttributeUnavailable)?,
        PrimitiveAttributeValue::Bool(value) => {
            if *value {
                target
                    .set_attribute(attr.name(), "")
                    .map_err(|_| PrimitiveError::AttributeUnavailable)?;
            } else {
                let _ = target.remove_attribute(attr.name());
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
struct AttributeDelta {
    remove: Vec<String>,
    set: Vec<PrimitiveAttribute>,
}

fn attribute_delta(
    previous: &[PrimitiveAttribute],
    next: &[PrimitiveAttribute],
) -> AttributeDelta {
    let previous = normalize_attributes(previous);
    let next = normalize_attributes(next);

    let previous_names: BTreeSet<&str> = previous.iter().map(|attr| attr.name()).collect();
    let next_names: BTreeSet<&str> = next.iter().map(|attr| attr.name()).collect();

    let remove = previous_names
        .difference(&next_names)
        .map(|name| (*name).to_string())
        .collect();

    AttributeDelta { remove, set: next }
}

fn normalize_attributes(attrs: &[PrimitiveAttribute]) -> Vec<PrimitiveAttribute> {
    let mut map: BTreeMap<String, PrimitiveAttributeValue> = BTreeMap::new();
    for attr in attrs {
        map.insert(attr.name.clone(), attr.value.clone());
    }
    map.into_iter()
        .map(|(name, value)| PrimitiveAttribute { name, value })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{attribute_delta, PrimitiveAttribute, PrimitiveAttributeValue};

    fn attr_str(name: &str, value: &str) -> PrimitiveAttribute {
        PrimitiveAttribute::string(name, value)
    }

    fn attr_bool(name: &str, value: bool) -> PrimitiveAttribute {
        PrimitiveAttribute::bool(name, value)
    }

    #[test]
    fn attribute_delta_dedupes_and_removes_missing() {
        let previous = vec![attr_str("data-state", "open"), attr_bool("hidden", true)];
        let next = vec![attr_str("data-state", "closed")];

        let delta = attribute_delta(&previous, &next);
        assert_eq!(delta.remove, vec!["hidden"]);
        assert_eq!(delta.set.len(), 1);
        assert_eq!(delta.set[0].name(), "data-state");
        assert_eq!(
            delta.set[0].value(),
            &PrimitiveAttributeValue::String("closed".to_string())
        );
    }

    #[test]
    fn attribute_delta_last_wins() {
        let next = vec![
            attr_str("data-state", "open"),
            attr_str("data-state", "closed"),
        ];
        let delta = attribute_delta(&[], &next);
        assert_eq!(delta.set.len(), 1);
        assert_eq!(delta.set[0].name(), "data-state");
        assert_eq!(
            delta.set[0].value(),
            &PrimitiveAttributeValue::String("closed".to_string())
        );
    }
}
