use crate::{
    Route,
    blog::{PostBlock, find_post, published_posts},
    components::{BlogVideo, CodeBlock, Comments, GcCalculator, ReactionBar},
    shared::{models::ReactionTarget, server_fns},
};
use dioxus::prelude::*;
use time::{Date, macros::format_description};

const POST_DATE: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[day padding:none] [month repr:short] [year]");
const ROW_DATE: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[day padding:none] [month repr:short]");

#[component]
pub fn Blog() -> Element {
    let posts = published_posts().collect::<Vec<_>>();
    let mut years = posts
        .iter()
        .map(|post| post.published.year())
        .collect::<Vec<_>>();
    years.dedup();

    rsx! {
        section { class: "flex flex-col gap-9",
            header { class: "flex flex-col gap-2",
                span { class: "label-mono", "// blog" }
                h1 { class: "heading-casual m-0 text-[30px] leading-[1.2]", "blog" }
            }
            if posts.is_empty() {
                p { class: "prose-font m-0 text-lg text-muted", "Nothing here yet." }
            }
            for year in years {
                div { class: "grid grid-cols-[60px_1fr] items-start gap-4 md:grid-cols-[72px_1fr]",
                    span { class: "label-mono pt-3", "{year}" }
                    div { class: "flex flex-col",
                        for post in posts.iter().copied().filter(|post| post.published.year() == year) {
                            Link {
                                key: "{post.slug}",
                                to: Route::BlogPost { slug: post.slug.to_string() },
                                class: "grid grid-cols-[1fr_auto] items-baseline gap-4 border-b border-line py-2.5 text-text no-underline hover:text-accent",
                                span { class: "flex flex-wrap items-baseline gap-2.5",
                                    span { class: "text-base", "{post.title}" }
                                    PostTags { tags: post.tags }
                                }
                                time { class: "label-mono whitespace-nowrap", datetime: post.published.to_string(), "{row_date(post.published)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn BlogPost(slug: String) -> Element {
    let Some(post) = find_post(&slug) else {
        return rsx! { crate::pages::NotFound { route: vec!["blog".to_string(), slug] } };
    };
    rsx! {
        article { class: "flex flex-col gap-7",
            Link { to: Route::Blog {}, class: "label-mono w-fit no-underline hover:text-accent", "← all posts" }
            header { class: "flex flex-col gap-2.5",
                h1 { class: "heading-casual m-0 text-pretty text-[34px] leading-[1.15] tracking-[-.015em]", "{post.title}" }
                div { class: "flex flex-wrap items-center gap-3",
                    span { class: "label-mono",
                        time { datetime: post.published.to_string(), "{post_date(post.published)}" }
                        " · {post.read_minutes} min read"
                    }
                    PostTags { tags: post.tags }
                }
            }
            div {
                div { class: "post-body",
                    for (index, block) in post.body.iter().enumerate() {
                        match block {
                            PostBlock::Html(html) => rsx! { crate::components::syntax::HighlightedHtml { key: "{post.slug}-html-{index}", html: *html } },
                            PostBlock::Code { language, title, highlighted, source } => rsx! {
                                CodeBlock {
                                    key: "{post.slug}-code-{index}",
                                    id: format!("blog-{}-code-{}", post.slug, index + 1),
                                    language: *language,
                                    title: *title,
                                    highlighted: *highlighted,
                                    source: *source,
                                }
                            },
                            PostBlock::GcCalculator => rsx! { GcCalculator { key: "{post.slug}-gc-{index}" } },
                            PostBlock::Video { src, title } => rsx! { BlogVideo { key: "{post.slug}-video-{index}", src: *src, title: *title } },
                        }
                    }
                }
            }
            // Public article content and controls must not wait for database-backed discussion.
            ErrorBoundary {
                handle_error: |_| rsx! { p { role: "alert", class: "label-mono", "Discussion is unavailable. Reload the page to retry." } },
                SuspenseBoundary {
                    fallback: |_| rsx! { p { role: "status", class: "label-mono", "Loading discussion…" } },
                    PostDiscussion { slug: post.slug }
                }
            }
        }
    }
}

#[component]
fn PostDiscussion(slug: &'static str) -> Element {
    let mut reactions = use_loader(move || server_fns::load_reactions(slug.to_string()))?;
    let viewer = use_loader(server_fns::get_user)?;
    rsx! {
        ReactionBar {
            target: ReactionTarget::Post { slug: slug.to_string() },
            counts: reactions.read().post.clone(),
            signed_in: viewer.read().is_some(),
            on_change: move |counts| reactions.write().post = counts,
        }
        Comments { slug, reactions, viewer }
    }
}

#[component]
fn PostTags(tags: &'static [&'static str]) -> Element {
    rsx! {
        if !tags.is_empty() {
            ul { class: "flex list-none flex-wrap gap-1.5 p-0", aria_label: "Tags",
                for tag in tags {
                    li { class: "tag", {tag.to_string()} }
                }
            }
        }
    }
}

fn post_date(date: Date) -> String {
    date.format(POST_DATE)
        .expect("the static date format is valid")
        .to_lowercase()
}

fn row_date(date: Date) -> String {
    date.format(ROW_DATE)
        .expect("the static date format is valid")
        .to_lowercase()
}
