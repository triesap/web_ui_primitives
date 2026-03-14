use leptos::html;
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomAttribute {
    name: String,
    value: DomAttributeValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomAttributeValue {
    String(String),
    Bool(bool),
}

impl DomAttribute {
    pub fn string(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: DomAttributeValue::String(value.into()),
        }
    }

    pub fn bool(name: impl Into<String>, value: bool) -> Self {
        Self {
            name: name.into(),
            value: DomAttributeValue::Bool(value),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &DomAttributeValue {
        &self.value
    }
}

#[cfg(target_arch = "wasm32")]
pub type DomEventHandler = Callback<web_sys::Event>;
#[cfg(not(target_arch = "wasm32"))]
pub type DomEventHandler = Callback<()>;

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Clone)]
pub struct DomEventBinding {
    name: &'static str,
    handler: DomEventHandler,
}

impl DomEventBinding {
    pub fn new(name: &'static str, handler: DomEventHandler) -> Self {
        Self { name, handler }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[cfg(target_arch = "wasm32")]
pub type DomTarget = web_sys::Element;
#[cfg(not(target_arch = "wasm32"))]
pub type DomTarget = ();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomBindingError {
    AttributeUnavailable,
}

pub type DomBindingResult<T> = Result<T, DomBindingError>;

#[derive(Clone)]
pub struct BoundElement<E>
where
    E: html::ElementType,
{
    node_ref: NodeRef<E>,
}

impl<E> BoundElement<E>
where
    E: html::ElementType,
{
    pub fn node_ref(&self) -> NodeRef<E> {
        self.node_ref
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn use_dom_bindings<E>(
    attrs: impl Into<Signal<Vec<DomAttribute>>>,
    events: Vec<DomEventBinding>,
) -> BoundElement<E>
where
    E: html::ElementType,
    E::Output: 'static,
{
    let _ = attrs.into();
    let _ = events;
    let node_ref = NodeRef::<E>::new();
    BoundElement { node_ref }
}

#[cfg(target_arch = "wasm32")]
pub fn use_dom_bindings<E>(
    attrs: impl Into<Signal<Vec<DomAttribute>>>,
    events: Vec<DomEventBinding>,
) -> BoundElement<E>
where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    let attrs = attrs.into();
    let node_ref = NodeRef::<E>::new();
    attach_dom_bindings(node_ref, attrs, events);
    BoundElement { node_ref }
}

#[cfg(target_arch = "wasm32")]
fn attach_dom_bindings<E>(
    node_ref: NodeRef<E>,
    attrs: Signal<Vec<DomAttribute>>,
    events: Vec<DomEventBinding>,
) where
    E: html::ElementType,
    E::Output: wasm_bindgen::JsCast + Clone + 'static,
{
    use send_wrapper::SendWrapper;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let prev_attrs: Rc<RefCell<Vec<DomAttribute>>> = Rc::new(RefCell::new(Vec::new()));
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
            if let Ok(normalized) =
                apply_dom_attribute_delta(&element_for_effect, &previous, &next)
            {
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
            let _ =
                element.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref());
            handles.push(EventHandle { name, closure });
        }

        if !handles.is_empty() {
            let cleanup_element = SendWrapper::new(element);
            let cleanup_handles = SendWrapper::new(handles);
            on_cleanup(move || {
                let element = cleanup_element.take();
                let handles = cleanup_handles.take();
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

pub fn apply_dom_attribute_delta(
    target: &DomTarget,
    previous: &[DomAttribute],
    next: &[DomAttribute],
) -> DomBindingResult<Vec<DomAttribute>> {
    let AttributeDelta { remove, set } = dom_attribute_delta(previous, next);

    #[cfg(target_arch = "wasm32")]
    {
        for name in remove {
            let _ = target.remove_attribute(&name);
        }

        for attr in &set {
            apply_dom_attribute(target, attr)?;
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
fn apply_dom_attribute(target: &DomTarget, attr: &DomAttribute) -> DomBindingResult<()> {
    match attr.value() {
        DomAttributeValue::String(value) => target
            .set_attribute(attr.name(), value)
            .map_err(|_| DomBindingError::AttributeUnavailable)?,
        DomAttributeValue::Bool(value) => {
            if *value {
                target
                    .set_attribute(attr.name(), "")
                    .map_err(|_| DomBindingError::AttributeUnavailable)?;
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
    set: Vec<DomAttribute>,
}

fn dom_attribute_delta(previous: &[DomAttribute], next: &[DomAttribute]) -> AttributeDelta {
    let previous = normalize_dom_attributes(previous);
    let next = normalize_dom_attributes(next);

    let previous_names: BTreeSet<&str> = previous.iter().map(|attr| attr.name()).collect();
    let next_names: BTreeSet<&str> = next.iter().map(|attr| attr.name()).collect();

    let remove = previous_names
        .difference(&next_names)
        .map(|name| (*name).to_string())
        .collect();

    AttributeDelta { remove, set: next }
}

fn normalize_dom_attributes(attrs: &[DomAttribute]) -> Vec<DomAttribute> {
    let mut map: BTreeMap<String, DomAttributeValue> = BTreeMap::new();
    for attr in attrs {
        map.insert(attr.name.clone(), attr.value.clone());
    }
    map.into_iter()
        .map(|(name, value)| DomAttribute { name, value })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{DomAttribute, DomAttributeValue, dom_attribute_delta};

    fn attr_str(name: &str, value: &str) -> DomAttribute {
        DomAttribute::string(name, value)
    }

    fn attr_bool(name: &str, value: bool) -> DomAttribute {
        DomAttribute::bool(name, value)
    }

    #[test]
    fn attribute_delta_dedupes_and_removes_missing() {
        let previous = vec![attr_str("data-state", "open"), attr_bool("hidden", true)];
        let next = vec![attr_str("data-state", "closed")];

        let delta = dom_attribute_delta(&previous, &next);
        assert_eq!(delta.remove, vec!["hidden"]);
        assert_eq!(delta.set.len(), 1);
        assert_eq!(delta.set[0].name(), "data-state");
        assert_eq!(
            delta.set[0].value(),
            &DomAttributeValue::String("closed".to_string())
        );
    }

    #[test]
    fn attribute_delta_last_wins() {
        let next = vec![
            attr_str("data-state", "open"),
            attr_str("data-state", "closed"),
        ];
        let delta = dom_attribute_delta(&[], &next);
        assert_eq!(delta.set.len(), 1);
        assert_eq!(delta.set[0].name(), "data-state");
        assert_eq!(
            delta.set[0].value(),
            &DomAttributeValue::String("closed".to_string())
        );
    }
}
