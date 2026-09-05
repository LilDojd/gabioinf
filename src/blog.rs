//! Blog posts generated and validated by `build.rs`.

use time::Date;

#[cfg(all(feature = "test-content", not(debug_assertions), not(test)))]
compile_error!("test-content is only for debug browser tests, not production builds");

// A catalog need not use every allowlisted component.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostBlock {
    Html(&'static str),
    /// Raw source and 1-based lines to emphasize; syntax highlighting loads in the browser.
    Code {
        language: Option<&'static str>,
        title: Option<&'static str>,
        highlighted: &'static [usize],
        source: &'static str,
    },
    GcCalculator,
    Video {
        src: &'static str,
        title: Option<&'static str>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Post {
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub published: Date,
    pub updated: Option<Date>,
    pub tags: &'static [&'static str],
    pub read_minutes: usize,
    pub body: &'static [PostBlock],
}

impl Post {
    #[cfg(feature = "server")]
    pub fn last_modified(self) -> Date {
        self.updated.unwrap_or(self.published)
    }
}

include!(concat!(env!("OUT_DIR"), "/blog_posts.rs"));

pub fn published_posts() -> impl Iterator<Item = &'static Post> + Clone {
    POSTS.iter()
}

pub fn find_post(slug: &str) -> Option<&'static Post> {
    // ponytail: a linear scan is ideal for a small personal blog; index it only if profiling says otherwise.
    published_posts().find(|post| post.slug == slug)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn generated_catalog_is_valid_and_sorted() {
        let mut slugs = HashSet::new();
        for post in POSTS {
            assert!(slugs.insert(post.slug), "duplicate slug: {}", post.slug);
            assert!(!post.title.is_empty());
            assert!(!post.description.is_empty());
            assert!(post.read_minutes > 0);
            assert!(!post.body.is_empty());
            assert!(post.updated.is_none_or(|updated| updated >= post.published));
        }
        assert!(POSTS.windows(2).all(|posts| {
            posts[0].published > posts[1].published
                || (posts[0].published == posts[1].published && posts[0].slug <= posts[1].slug)
        }));
    }

    #[test]
    fn drafts_are_not_public() {
        assert!(
            !DRAFT_SLUGS.is_empty(),
            "the catalog includes a draft fixture"
        );
        assert!(DRAFT_SLUGS.iter().all(|slug| find_post(slug).is_none()));
    }
}
