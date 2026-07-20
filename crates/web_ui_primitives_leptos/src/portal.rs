use leptos::html;
use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Attribute applied to the DOM subtree owned by [`Portal`].
pub const PORTAL_CONTAINER_ATTRIBUTE: &str = "data-web-ui-portal";

/// Mount target used by [`Portal`].
///
/// On wasm this is a concrete DOM element. On non-wasm targets it becomes `()`
/// because no live DOM mount point exists.
#[cfg(target_arch = "wasm32")]
pub type PortalMount = web_sys::Element;

/// Mount target used by [`Portal`].
///
/// On wasm this is a concrete DOM element. On non-wasm targets it becomes `()`
/// because no live DOM mount point exists.
#[cfg(not(target_arch = "wasm32"))]
pub type PortalMount = ();

/// Renders one deterministic portal container on every delivery target.
///
/// SSR emits the owned container inline so hydration can attach to the same
/// subtree. In the browser, the hydrated or CSR-created container is moved to
/// `mount`, or to `document.body` when no explicit mount is supplied. Moving
/// the existing container preserves child identity, focus, and event
/// bindings. Set `reparent` to `false` to retain the inline container.
#[component]
pub fn Portal(
    #[prop(optional)] mount: Option<PortalMount>,
    #[prop(default = true)] reparent: bool,
    children: ChildrenFn,
) -> impl IntoView {
    #[cfg(all(target_arch = "wasm32", feature = "csr"))]
    if reparent {
        return mount_csr_portal(mount, children);
    }

    let container = NodeRef::<html::Div>::new();

    #[cfg(all(target_arch = "wasm32", not(feature = "csr")))]
    attach_portal_container(container, mount, reparent);

    #[cfg(not(target_arch = "wasm32"))]
    let _ = (mount, reparent);

    view! {
        <div node_ref=container data-web-ui-portal="">
            {children()}
        </div>
    }
    .into_any()
}

#[cfg(all(target_arch = "wasm32", feature = "csr"))]
fn mount_csr_portal(mount: Option<PortalMount>, children: ChildrenFn) -> AnyView {
    use leptos::mount::mount_to;
    use send_wrapper::SendWrapper;

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return ().into_any();
    };
    let Some(target) = mount.or_else(|| document.body().map(Into::into)) else {
        return ().into_any();
    };
    let Ok(container) = document.create_element("div") else {
        return ().into_any();
    };
    let _ = container.set_attribute(PORTAL_CONTAINER_ATTRIBUTE, "");
    let html_container = container.clone().unchecked_into::<web_sys::HtmlElement>();
    let _ = target.append_child(&container);

    let handle = SendWrapper::new((mount_to(html_container, move || children()), container));
    on_cleanup(move || {
        let (handle, container) = handle.take();
        drop(handle);
        if let Some(parent) = container.parent_node() {
            let _ = parent.remove_child(&container);
        }
    });

    ().into_any()
}

#[cfg(all(target_arch = "wasm32", not(feature = "csr")))]
fn attach_portal_container(
    container_ref: NodeRef<html::Div>,
    mount: Option<PortalMount>,
    reparent: bool,
) {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use send_wrapper::SendWrapper;

    let active = Arc::new(AtomicBool::new(true));
    let active_for_effect = Arc::clone(&active);
    let mount = SendWrapper::new(mount);

    let effect = RenderEffect::new(move |_| {
        if !reparent {
            return;
        }
        let Some(container) = container_ref
            .get()
            .and_then(|container| container.dyn_into::<web_sys::Element>().ok())
        else {
            return;
        };
        let target = mount
            .as_ref()
            .cloned()
            .or_else(|| document().body().map(Into::into));
        let Some(target) = target else {
            return;
        };
        let active_for_move = Arc::clone(&active_for_effect);

        let move_container = move || {
            if !active_for_move.load(Ordering::Acquire)
                || target == container
                || container.contains(Some(&target))
            {
                return;
            }
            let active_descendant = document()
                .active_element()
                .filter(|active| container.contains(Some(active)));
            let _ = target.append_child(&container);
            if let Some(active_descendant) =
                active_descendant.and_then(|active| active.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let _ = active_descendant.focus();
            }
        };

        let during_hydration =
            Owner::current_shared_context().is_some_and(|context| context.during_hydration());
        if during_hydration {
            queue_microtask(move_container);
        } else {
            move_container();
        }
    });

    on_cleanup(move || {
        drop(effect);
        active.store(false, Ordering::Release);
        let Some(container) = container_ref
            .get_untracked()
            .and_then(|container| container.dyn_into::<web_sys::Element>().ok())
        else {
            return;
        };
        let Some(parent) = container.parent_node() else {
            return;
        };
        let _ = parent.remove_child(&container);
    });
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn portal_mount_is_unit_on_host_targets() {
        assert_eq!(core::mem::size_of::<super::PortalMount>(), 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "ssr"))]
    #[test]
    fn portal_ssr_owns_one_deterministic_inline_container() {
        use leptos::prelude::*;

        let html = view! {
            <super::Portal>
                <span data-portal-child="one">"Portaled"</span>
            </super::Portal>
        }
        .to_html();

        assert_eq!(html.matches(super::PORTAL_CONTAINER_ATTRIBUTE).count(), 1);
        assert_eq!(html.matches("data-portal-child=\"one\"").count(), 1);
        assert!(html.contains(">Portaled</span>"));
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "ssr"))]
    #[test]
    fn nested_portal_ssr_preserves_distinct_owned_subtrees() {
        use leptos::prelude::*;

        let html = view! {
            <super::Portal>
                <span>"Outer"</span>
                <super::Portal>
                    <span>"Inner"</span>
                </super::Portal>
            </super::Portal>
        }
        .to_html();

        assert_eq!(html.matches(super::PORTAL_CONTAINER_ATTRIBUTE).count(), 2);
        assert_eq!(html.matches(">Outer</span>").count(), 1);
        assert_eq!(html.matches(">Inner</span>").count(), 1);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "ssr"))]
    #[test]
    fn disabled_reparenting_keeps_the_same_ssr_contract() {
        use leptos::prelude::*;

        let html = view! {
            <super::Portal reparent=false>
                <span>"Inline"</span>
            </super::Portal>
        }
        .to_html();

        assert_eq!(html.matches(super::PORTAL_CONTAINER_ATTRIBUTE).count(), 1);
        assert!(html.contains(">Inline</span>"));
    }
}
