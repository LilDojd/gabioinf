//! Public blog feeds and sitemap.

use crate::blog::published_posts;
use axum::{
    Router, extract::State, http::header::CONTENT_TYPE, response::IntoResponse, routing::get,
};

#[derive(Clone)]
struct BlogState {
    origin: String,
}

pub fn router(domain: &str) -> Router {
    Router::new()
        .route("/feed.xml", get(feed))
        .route("/sitemap.xml", get(sitemap))
        .route("/robots.txt", get(robots))
        .with_state(BlogState {
            origin: site_origin(domain),
        })
}

async fn feed(State(state): State<BlogState>) -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        atom_feed(&state.origin),
    )
}

async fn sitemap(State(state): State<BlogState>) -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/xml; charset=utf-8")],
        sitemap_xml(&state.origin),
    )
}

async fn robots(State(state): State<BlogState>) -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots_txt(&state.origin),
    )
}

fn robots_txt(origin: &str) -> String {
    format!("User-agent: *\nAllow: /\nDisallow: /v1/\n\nSitemap: {origin}/sitemap.xml\n")
}

fn site_origin(domain: &str) -> String {
    let domain = domain.trim_end_matches('/');
    if domain.starts_with("http://") || domain.starts_with("https://") {
        domain.to_string()
    } else {
        format!("https://{domain}")
    }
}

fn atom_feed(origin: &str) -> String {
    let origin = escape_xml(origin);
    let updated = published_posts()
        .map(|post| post.last_modified())
        .max()
        .map_or_else(|| "1970-01-01".to_string(), |date| date.to_string());
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\"><title>George Andreev's blog</title><id>{origin}/blog</id><link href=\"{origin}/feed.xml\" rel=\"self\"/><link href=\"{origin}/blog\"/><updated>{updated}T00:00:00Z</updated><author><name>George Andreev</name></author>"
    );

    for post in published_posts() {
        let url = format!("{origin}/blog/{}", post.slug);
        xml.push_str(&format!(
            "<entry><title>{}</title><id>{url}</id><link href=\"{url}\"/><published>{}T00:00:00Z</published><updated>{}T00:00:00Z</updated><summary>{}</summary>",
            escape_xml(post.title),
            post.published,
            post.last_modified(),
            escape_xml(post.description),
        ));
        for tag in post.tags {
            xml.push_str(&format!("<category term=\"{}\"/>", escape_xml(tag)));
        }
        xml.push_str("</entry>");
    }
    xml.push_str("</feed>\n");
    xml
}

fn sitemap_xml(origin: &str) -> String {
    let origin = escape_xml(origin);
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">",
    );
    for path in ["/", "/blog", "/projects", "/about", "/guestbook"] {
        xml.push_str(&format!("<url><loc>{origin}{path}</loc></url>"));
    }
    for post in published_posts() {
        xml.push_str(&format!(
            "<url><loc>{origin}/blog/{}</loc><lastmod>{}</lastmod></url>",
            post.slug,
            post.last_modified(),
        ));
    }
    xml.push_str("</urlset>\n");
    xml
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_control() && !matches!(character, '\t' | '\n' | '\r') {
            continue;
        }
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_site_origin() {
        assert_eq!(site_origin("gabioinf.dev"), "https://gabioinf.dev");
        assert_eq!(
            site_origin("http://localhost:8080/"),
            "http://localhost:8080"
        );
    }

    #[test]
    fn feed_is_valid_at_its_boundaries_and_has_an_author() {
        let feed = atom_feed("https://example.test");

        assert!(feed.starts_with("<?xml"));
        assert!(feed.contains("<author><name>George Andreev</name></author>"));
        assert!(feed.ends_with("</feed>\n"));
    }

    #[test]
    fn sitemap_contains_public_routes() {
        let sitemap = sitemap_xml("https://example.test");

        assert!(sitemap.contains("https://example.test/blog"));
        assert!(sitemap.ends_with("</urlset>\n"));
    }

    #[test]
    fn robots_points_to_the_sitemap() {
        let robots = robots_txt("https://example.test");

        assert!(robots.contains("Sitemap: https://example.test/sitemap.xml"));
        assert!(robots.contains("Disallow: /v1/"));
    }

    #[test]
    fn xml_escaping_removes_invalid_controls() {
        assert_eq!(escape_xml("<A & \"B\">\0"), "&lt;A &amp; &quot;B&quot;&gt;");
    }
}
