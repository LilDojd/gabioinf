//! Giscus comments backed by this repository's GitHub Discussions.

use dioxus::prelude::*;

const DISCUSSIONS_URL: &str = "https://github.com/LilDojd/gabioinf/discussions";

#[cfg(any(feature = "web", test))]
const MOUNT_GISCUS: &str = r#"
const container = document.getElementById('giscus-comments');
if (container) {
    container.replaceChildren();
    const script = document.createElement('script');
    script.src = 'https://giscus.app/client.js';
    script.async = true;
    script.crossOrigin = 'anonymous';
    for (const [key, value] of Object.entries(container.dataset)) {
        if (key !== 'commentId') script.dataset[key] = value;
    }
    script.dataset.term = container.dataset.commentId;
    container.appendChild(script);
}
"#;

#[cfg(any(feature = "web", test))]
const UNMOUNT_GISCUS: &str = r#"
document.getElementById('giscus-comments')?.replaceChildren();
"#;

#[component]
pub fn Comments(comment_id: &'static str) -> Element {
    use_effect(use_reactive!(|comment_id| {
        let _ = comment_id;
        #[cfg(feature = "web")]
        {
            _ = document::eval(MOUNT_GISCUS);
        }
    }));
    use_drop(|| {
        #[cfg(feature = "web")]
        {
            _ = document::eval(UNMOUNT_GISCUS);
        }
    });

    rsx! {
        section {
            class: "mt-8 border-t border-line pt-7",
            aria_labelledby: "comments-heading",
            h2 { id: "comments-heading", class: "label-mono m-0", "// comments" }
            p { class: "prose-font mb-6 mt-3 text-base text-muted",
                "Comments are public and hosted in "
                a {
                    class: "link-dashed",
                    href: DISCUSSIONS_URL,
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "GitHub Discussions"
                }
                ". A GitHub account is required."
            }
            div {
                key: "{comment_id}",
                id: "giscus-comments",
                class: "giscus min-h-24 rounded-md",
                aria_label: "Public article comments",
                "data-comment-id": comment_id,
                "data-repo": "LilDojd/gabioinf",
                "data-repo-id": "R_kgDOMbIfMQ",
                "data-category": "Announcements",
                "data-category-id": "DIC_kwDOMbIfMc4DEwAo",
                "data-mapping": "specific",
                "data-strict": "1",
                "data-reactions-enabled": "1",
                "data-emit-metadata": "0",
                "data-input-position": "bottom",
                "data-theme": "dark",
                "data-lang": "en",
                "data-loading": "lazy",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_is_specific_and_lazy() {
        assert!(MOUNT_GISCUS.contains("dataset.term"));
        assert!(MOUNT_GISCUS.contains("giscus.app/client.js"));
        assert!(UNMOUNT_GISCUS.contains("replaceChildren"));
    }
}
