use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use leptos::portal::Portal as LeptosPortal;

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
        match mount {
            Some(mount) => view! { <LeptosPortal mount=mount>{children()}</LeptosPortal> },
            None => view! { <LeptosPortal>{children()}</LeptosPortal> },
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mount;
        children()
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
