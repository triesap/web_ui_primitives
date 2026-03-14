use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use leptos::portal::Portal as LeptosPortal;

#[cfg(target_arch = "wasm32")]
pub type PortalMount = web_sys::Element;

#[cfg(not(target_arch = "wasm32"))]
pub type PortalMount = ();

#[component]
pub fn Portal(
    #[prop(optional)] mount: Option<PortalMount>,
    children: ChildrenFn,
) -> impl IntoView {
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
    #[test]
    fn portal_availability_matches_target() {
        assert!(!cfg!(target_arch = "wasm32"));
    }
}
