use crate::{
    Route,
    blog::{Post, PostBlock, find_post, published_posts},
    components::{BlogVideo, Comments, GcCalculator},
};
use dioxus::prelude::*;
use time::{Date, macros::format_description};

const DISPLAY_DATE: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[day padding:none] [month repr:long] [year]");

#[component]
pub fn Blog() -> Element {
    let posts = published_posts().collect::<Vec<_>>();

    rsx! {
        main { class: "w-full",
            header { class: "mb-12 max-w-2xl",
                div { class: "flex flex-wrap items-baseline justify-between gap-4",
                    h1 { class: "text-4xl font-bold tracking-tight text-stone-100", "Blog" }
                    a {
                        class: "alien-link-muted text-sm focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-alien-green",
                        href: "/feed.xml",
                        "Atom feed"
                    }
                }
                p { class: "mt-4 text-lg leading-8 text-stone-300",
                    "Notes about bioinformatics, Rust, and useful experiments."
                }
            }

            if posts.is_empty() {
                p { class: "rounded-lg border border-onyx bg-jet p-6 text-stone-300",
                    "No posts published yet. Check back soon."
                }
            } else {
                ol { class: "space-y-6",
                    for post in posts {
                        BlogPostSummary { key: "{post.slug}", post }
                    }
                }
            }
        }
    }
}

#[component]
fn BlogPostSummary(post: &'static Post) -> Element {
    rsx! {
        li {
            article {
                class: "group rounded-lg border border-onyx bg-jet p-6 transition-colors hover:border-alien-green focus-within:border-alien-green",
                h2 { class: "text-2xl font-semibold text-stone-100",
                    Link {
                        class: "rounded-sm transition-colors group-hover:text-alien-green focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-alien-green",
                        to: Route::BlogPost { slug: post.slug.to_string() },
                        {post.title}
                    }
                }
                p { class: "mt-2 text-sm text-stone-400",
                    time {
                        datetime: post.published.to_string(),
                        {display_date(post.published)}
                    }
                    " · {post.read_minutes} min read"
                }
                p { class: "mt-4 leading-7 text-stone-300", {post.description} }
                PostTags { tags: post.tags }
            }
        }
    }
}

#[component]
pub fn BlogPost(slug: String) -> Element {
    let Some(post) = find_post(&slug) else {
        return rsx! {
            crate::pages::NotFound { route: vec!["blog".to_string(), slug] }
        };
    };

    rsx! {
        main { class: "w-full",
            article { class: "mx-auto max-w-3xl",
                header { class: "mb-10 border-b border-onyx pb-8",
                    Link {
                        class: "alien-link-muted inline-block rounded-sm text-sm focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-alien-green",
                        to: Route::Blog {},
                        "← All posts"
                    }
                    h1 { class: "mt-5 text-4xl font-bold leading-tight tracking-tight text-stone-100 md:text-5xl",
                        {post.title}
                    }
                    p { class: "mt-5 text-lg leading-8 text-stone-300", {post.description} }
                    p { class: "mt-5 text-sm text-stone-400",
                        "Published "
                        time {
                            datetime: post.published.to_string(),
                            {display_date(post.published)}
                        }
                        if let Some(updated) = post.updated.filter(|updated| *updated > post.published) {
                            " · Updated "
                            time {
                                datetime: updated.to_string(),
                                {display_date(updated)}
                            }
                        }
                        " · {post.read_minutes} min read"
                    }
                    PostTags { tags: post.tags }
                }
                div {
                    for block in post.body {
                        match block {
                            PostBlock::Html(html) => rsx! {
                                // HTML is generated from validated Markdown at build time.
                                div {
                                    class: "blog-markdown prose prose-invert prose-stone max-w-none",
                                    dangerous_inner_html: *html,
                                }
                            },
                            PostBlock::GcCalculator => rsx! { GcCalculator {} },
                            PostBlock::Video { src, title } => rsx! {
                                BlogVideo { src: *src, title: *title }
                            },
                        }
                    }
                }
                Comments { comment_id: post.slug }
            }
        }
    }
}

#[component]
fn PostTags(tags: &'static [&'static str]) -> Element {
    if tags.is_empty() {
        return rsx! {};
    }

    rsx! {
        ul { class: "mt-5 flex flex-wrap gap-2", aria_label: "Tags",
            for tag in tags {
                li {
                    class: "rounded-full border border-onyx bg-nasty-black px-3 py-1 text-xs text-stone-300",
                    {*tag}
                }
            }
        }
    }
}

fn display_date(date: Date) -> String {
    date.format(DISPLAY_DATE)
        .expect("the static date format is valid")
}
