use crate::{
    Route,
    blog::{PostBlock, find_post, published_posts},
    components::{BlogVideo, Comments, GcCalculator},
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
                h1 { class: "heading-casual m-0 text-[30px] leading-[1.2]", "random rambles" }
            }
            if posts.is_empty() {
                p { class: "prose-font m-0 text-lg text-muted", "No rambles yet. Check back soon." }
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
            Link { to: Route::Blog {}, class: "label-mono w-fit no-underline hover:text-accent", "← all rambles" }
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
            div { class: "post-body",
                for block in post.body {
                    match block {
                        PostBlock::Html(html) => rsx! { div { dangerous_inner_html: *html } },
                        PostBlock::GcCalculator => rsx! { GcCalculator {} },
                        PostBlock::Video { src, title } => rsx! { BlogVideo { src: *src, title: *title } },
                    }
                }
            }
            Comments { slug: post.slug }
        }
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
