//! Modal sibling-hiding helpers for headless overlays.

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};

#[cfg(target_arch = "wasm32")]
use web_sys::{Element, window};

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
struct HiddenElement {
    element: Element,
    prev_aria_hidden: Option<String>,
    prev_inert: bool,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct HiddenElement;

#[derive(Debug, Clone)]
struct ModalLayer {
    id: u64,
    hidden: Vec<HiddenElement>,
}

#[derive(Debug, Default)]
struct ModalState {
    next_id: u64,
    layers: Vec<ModalLayer>,
}

/// Errors that can occur while applying modal sibling hiding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalError {
    WindowUnavailable,
    DocumentUnavailable,
    BodyUnavailable,
    AttributeUnavailable,
}

/// Result type used by modal helpers.
pub type ModalResult<T> = Result<T, ModalError>;

/// Modal root target used by [`modal_hide_siblings`].
///
/// On wasm this is a concrete DOM element. On non-wasm targets it becomes `()`
/// because there is no live DOM tree to mutate.
#[cfg(target_arch = "wasm32")]
pub type ModalTarget = Element;

/// Modal root target used by [`modal_hide_siblings`].
///
/// On wasm this is a concrete DOM element. On non-wasm targets it becomes `()`
/// because there is no live DOM tree to mutate.
#[cfg(not(target_arch = "wasm32"))]
pub type ModalTarget = ();

/// RAII guard returned by [`modal_hide_siblings`].
///
/// Dropping the guard restores any siblings hidden for that modal layer unless
/// another active layer still requires them to stay hidden.
#[derive(Debug)]
pub struct ModalGuard {
    id: u64,
    active: bool,
}

impl Drop for ModalGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = modal_restore(self.id);
            self.active = false;
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static MODAL_STATE: RefCell<ModalState> = RefCell::new(ModalState::default());
}

#[cfg(not(target_arch = "wasm32"))]
static MODAL_STATE: OnceLock<Mutex<ModalState>> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
fn modal_state_with<T>(f: impl FnOnce(&mut ModalState) -> T) -> T {
    MODAL_STATE.with(|state| f(&mut state.borrow_mut()))
}

#[cfg(not(target_arch = "wasm32"))]
fn modal_state_with<T>(f: impl FnOnce(&mut ModalState) -> T) -> T {
    let mut state = MODAL_STATE
        .get_or_init(|| Mutex::new(ModalState::default()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut state)
}

/// Hides siblings outside the modal branch and returns a restoration guard.
///
/// On wasm, this walks ancestor levels from `root` up to `body`, applying
/// `aria-hidden` and `inert` to siblings outside the active branch.
///
/// On non-wasm targets, this records a logical modal layer without mutating a
/// DOM tree so tests and host builds remain deterministic.
pub fn modal_hide_siblings(root: &ModalTarget) -> ModalResult<ModalGuard> {
    let id = modal_state_with(|state| {
        let id = state.next_id;
        state.next_id = state.next_id.saturating_add(1);
        let hidden = modal_collect_hidden(root)?;
        state.layers.push(ModalLayer { id, hidden });
        Ok(id)
    })?;
    Ok(ModalGuard { id, active: true })
}

/// Restores the hidden siblings for a modal layer id.
///
/// If the id is unknown, this is a no-op. On wasm, siblings are only restored
/// when no remaining modal layer still hides them.
pub fn modal_restore(id: u64) -> ModalResult<()> {
    modal_state_with(|state| {
        let index = state.layers.iter().position(|layer| layer.id == id);
        let Some(index) = index else {
            return Ok(());
        };
        let removed = state.layers.remove(index);
        modal_restore_hidden(&state.layers, removed.hidden)?;
        Ok(())
    })
}

#[cfg(target_arch = "wasm32")]
fn modal_collect_hidden(root: &Element) -> ModalResult<Vec<HiddenElement>> {
    let mut hidden = Vec::new();
    let window = window().ok_or(ModalError::WindowUnavailable)?;
    let document = window.document().ok_or(ModalError::DocumentUnavailable)?;
    let body = document.body().ok_or(ModalError::BodyUnavailable)?;
    let mut current = root.clone();

    loop {
        let Some(parent) = current.parent_element() else {
            break;
        };
        let children = parent.children();
        let Some(active_index) = modal_child_index(&children, &current) else {
            break;
        };
        for index in modal_hidden_sibling_indexes(active_index, children.length()) {
            let Some(child) = children.item(index) else {
                continue;
            };
            modal_hide_element(&child, &mut hidden)?;
        }
        if parent.is_same_node(Some(body.as_ref())) {
            break;
        }
        current = parent;
    }
    Ok(hidden)
}

#[cfg(not(target_arch = "wasm32"))]
fn modal_collect_hidden(_root: &ModalTarget) -> ModalResult<Vec<HiddenElement>> {
    Ok(Vec::new())
}

#[cfg(target_arch = "wasm32")]
fn modal_child_index(children: &web_sys::HtmlCollection, current: &Element) -> Option<u32> {
    for index in 0..children.length() {
        let child = children.item(index)?;
        if child.is_same_node(Some(current)) {
            return Some(index);
        }
    }
    None
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn modal_hidden_sibling_indexes(active_index: u32, sibling_count: u32) -> Vec<u32> {
    (0..sibling_count)
        .filter(|index| *index != active_index)
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn modal_hide_element(child: &Element, hidden: &mut Vec<HiddenElement>) -> ModalResult<()> {
    if hidden
        .iter()
        .any(|item| item.element.is_same_node(Some(child)))
    {
        return Ok(());
    }

    let prev_aria_hidden = child.get_attribute("aria-hidden");
    let prev_inert = child.has_attribute("inert");
    child
        .set_attribute("aria-hidden", "true")
        .map_err(|_| ModalError::AttributeUnavailable)?;
    child
        .set_attribute("inert", "")
        .map_err(|_| ModalError::AttributeUnavailable)?;
    hidden.push(HiddenElement {
        element: child.clone(),
        prev_aria_hidden,
        prev_inert,
    });
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn modal_restore_hidden(layers: &[ModalLayer], hidden: Vec<HiddenElement>) -> ModalResult<()> {
    for item in hidden {
        if modal_is_hidden_by_layers(layers, &item.element) {
            continue;
        }
        match item.prev_aria_hidden {
            Some(value) => item
                .element
                .set_attribute("aria-hidden", &value)
                .map_err(|_| ModalError::AttributeUnavailable)?,
            None => item
                .element
                .remove_attribute("aria-hidden")
                .map_err(|_| ModalError::AttributeUnavailable)?,
        }
        if item.prev_inert {
            item.element
                .set_attribute("inert", "")
                .map_err(|_| ModalError::AttributeUnavailable)?;
        } else {
            item.element
                .remove_attribute("inert")
                .map_err(|_| ModalError::AttributeUnavailable)?;
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn modal_restore_hidden(_layers: &[ModalLayer], _hidden: Vec<HiddenElement>) -> ModalResult<()> {
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn modal_is_hidden_by_layers(layers: &[ModalLayer], element: &Element) -> bool {
    layers.iter().any(|layer| {
        layer
            .hidden
            .iter()
            .any(|item| item.element.is_same_node(Some(element)))
    })
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[cfg(not(target_arch = "wasm32"))]
fn modal_is_hidden_by_layers(_layers: &[ModalLayer], _element: &ModalTarget) -> bool {
    false
}

#[cfg(test)]
fn modal_layer_count_for_test() -> usize {
    modal_state_with(|state| state.layers.len())
}

#[cfg(test)]
mod tests {
    use super::{modal_hidden_sibling_indexes, modal_hide_siblings, modal_layer_count_for_test};

    #[test]
    fn modal_guard_tracks_layers() {
        assert_eq!(modal_layer_count_for_test(), 0);
        let guard = modal_hide_siblings(&()).expect("guard");
        assert_eq!(modal_layer_count_for_test(), 1);
        drop(guard);
        assert_eq!(modal_layer_count_for_test(), 0);
    }

    #[test]
    fn modal_hidden_sibling_indexes_skip_active_branch() {
        assert_eq!(modal_hidden_sibling_indexes(1, 4), vec![0, 2, 3]);
    }
}
