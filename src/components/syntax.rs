//! Progressive Tree-sitter highlighting. Plain code is always rendered first.
use dioxus::prelude::*;

#[cfg(feature = "web")]
pub(super) async fn highlight_code(language: Option<&str>, source: &str) -> Option<String> {
    let eval = document::eval(
        r#"const [url, language, source] = await dioxus.recv();
        const { highlightCode } = await import(url);
        return await highlightCode(language, source);"#,
    );
    if eval
        .send((
            asset!("/assets/code_highlighting.js").to_string(),
            language,
            source,
        ))
        .is_err()
    {
        return None;
    }
    eval.join::<Option<String>>().await.ok().flatten()
}

/// For already-sanitized Markdown HTML only. Dioxus treats the inner DOM as opaque.
#[component]
pub(crate) fn HighlightedHtml(html: String, #[props(default)] class: String) -> Element {
    rsx! {
        div { key: "{html}", class, dangerous_inner_html: html, onmounted: highlight_markdown }
    }
}

fn highlight_markdown(event: MountedEvent) {
    #[cfg(feature = "web")]
    {
        use web_sys::wasm_bindgen::JsCast;
        let Some(root) = event.data().downcast::<web_sys::Element>().cloned() else {
            return;
        };
        let Ok(nodes) = root.query_selector_all("pre code") else {
            return;
        };
        for index in 0..nodes.length() {
            let Some(code) = nodes
                .item(index)
                .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
            else {
                continue;
            };
            let source = code.text_content().unwrap_or_default();
            let language = code
                .class_name()
                .split_whitespace()
                .find_map(|class| class.strip_prefix("language-").map(str::to_string));
            spawn(async move {
                let Some(html) = highlight_code(language.as_deref(), &source).await else {
                    return;
                };
                while code.is_connected()
                    && web_sys::window()
                        .and_then(|window| window.get_selection().ok().flatten())
                        .is_some_and(|selection| {
                            !selection.is_collapsed()
                                && selection
                                    .contains_node_with_allow_partial_containment(&code, true)
                                    .unwrap_or(false)
                        })
                {
                    wasmtimer::tokio::sleep(std::time::Duration::from_millis(100)).await;
                }
                if code.is_connected() && code.text_content().as_deref() == Some(source.as_str()) {
                    code.set_inner_html(&html);
                }
            });
        }
    }
    #[cfg(not(feature = "web"))]
    let _ = event;
}
