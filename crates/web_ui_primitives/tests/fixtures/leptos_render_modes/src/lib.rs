use leptos::prelude::*;
use web_ui_primitives::leptos::{FocusScope, Presence};

#[component]
pub fn PrimitiveProbe() -> impl IntoView {
    let present = Signal::derive(|| true);

    view! {
        <FocusScope trapped=false auto_focus=false return_focus=false>
            <Presence present=present>
                <span data-primitive-probe="ready">"ready"</span>
            </Presence>
        </FocusScope>
    }
}

#[cfg(feature = "ssr")]
pub fn render_primitive_probe() -> String {
    view! { <PrimitiveProbe/> }.to_html()
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ssr")]
    #[test]
    fn native_ssr_renders_the_shared_primitive_tree() {
        let html = super::render_primitive_probe();
        assert!(html.contains("data-primitive-probe=\"ready\""));
        assert!(html.contains(">ready</span>"));
    }
}
