use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

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

#[component]
/// Renders children in a portal when running in the browser.
///
/// On non-wasm targets this falls back to rendering `children` inline and
/// ignores `mount`.
pub fn Portal(#[prop(optional)] mount: Option<PortalMount>, children: ChildrenFn) -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
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
        let html_container = container.clone().unchecked_into::<web_sys::HtmlElement>();
        let _ = target.append_child(&container);

        let handle = SendWrapper::new((
            mount_to(html_container, move || children()),
            target,
            container,
        ));
        on_cleanup(move || {
            let (handle, target, container) = handle.take();
            drop(handle);
            let _ = target.remove_child(&container);
        });

        ().into_any()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mount;
        children().into_any()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    use super::PortalMount;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn portal_mount_is_unit_on_host_targets() {
        assert_eq!(core::mem::size_of::<PortalMount>(), 0);
    }
}
