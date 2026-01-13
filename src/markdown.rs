use dioxus::prelude::*;
use dioxus_logger::tracing::error;
use markdown::{ParseOptions, mdast::Node, to_mdast};
#[derive(Copy, Clone, Default, PartialEq)]
#[allow(dead_code)]
pub enum MarkdownType {
    Github,
    #[default]
    Normal,
    Mdx,
}
impl MarkdownType {
    fn to_settings(self) -> ParseOptions {
        match self {
            MarkdownType::Github => ParseOptions::gfm(),
            MarkdownType::Normal => ParseOptions::mdx(),
            MarkdownType::Mdx => ParseOptions::default(),
        }
    }
}
#[component(no_case_check)]
pub fn Markdown(
    value: String,
    #[props(default)] class: String,
    #[props(default)] style: String,
    #[props(default)] md_type: MarkdownType,
    #[props(extends = div)] rest_attributes: Vec<Attribute>,
) -> Element {
    let nodes = match to_mdast(&value, &md_type.to_settings()) {
        Ok(nodes) => nodes,
        Err(_) => unreachable!(),
    };
    rsx! {
        div {..rest_attributes,
            style {}
            {expand_node(&nodes)}
        }
    }
}
fn expand_node(node: &Node) -> Element {
    match node {
        Node::Root(root) => {
            rsx! {
                {root.children.iter().map(expand_node)}
            }
        }
        Node::Blockquote(bq) => {
            rsx! {
                blockquote { {bq.children.iter().map(expand_node)} }
            }
        }
        Node::FootnoteDefinition(fnd) => {
            rsx! {
                p { id: if let Some(id) = &fnd.label { "{id}" },
                    "[{fnd.identifier}]:"
                    {fnd.children.iter().map(expand_node)}
                }
            }
        }
        Node::List(list) => {
            let children = list.children.iter().map(expand_node);
            let start = list.start.filter(|start| list.ordered && *start != 1);
            if list.ordered {
                rsx! {
                    ol { start: if let Some(start) = start { "{start}" }, {children} }
                }
            } else {
                rsx! {
                    ul { {children} }
                }
            }
        }
        Node::Toml(toml) => {
            rsx! {
                pre { "{toml.value}" }
            }
        }
        Node::Yaml(yaml) => {
            rsx! {
                pre { "{yaml.value}" }
            }
        }
        Node::Break(_) => rsx!(
            br {}
        ),
        Node::InlineCode(ilc) => {
            rsx! {
                code { "{ilc.value}" }
            }
        }
        Node::Delete(del) => {
            rsx! {
                del { {del.children.iter().map(expand_node)} }
            }
        }
        Node::Emphasis(emp) => {
            rsx! {
                em { {emp.children.iter().map(expand_node)} }
            }
        }
        Node::FootnoteReference(fnref) => {
            rsx! {
                a { href: if let Some(lab) = &fnref.label { "#{lab}" }, "[{fnref.identifier}]" }
            }
        }
        Node::Html(html) => {
            rsx! {
                div { dangerous_inner_html: "{html.value}" }
            }
        }
        Node::Image(img) => {
            rsx! {
                img {
                    src: "{img.url}",
                    alt: "{img.alt}",
                    title: if let Some(title) = &img.title { "{title}" },
                }
            }
        }
        Node::ImageReference(ir) => {
            rsx! {
                img { src: "{ir.identifier}", alt: "{ir.alt}" }
                if let Some(label) = &ir.label {
                    "{label}"
                }
            }
        }
        Node::Link(link) => {
            rsx!(
                a {
                    href: "{link.url}",
                    class: "alien-link",
                    target: "_blank",
                    title: if let Some(title) = &link.title { "{title}" },
                    {link.children.iter().map(expand_node)}
                }
            )
        }
        Node::LinkReference(lr) => {
            rsx!(
                a { aria_label: if let Some(label) = &lr.label { "{label}" },
                    "{lr.identifier}"
                    {lr.children.iter().map(expand_node)}
                }
            )
        }
        Node::Strong(strong) => {
            rsx!(
                strong { {strong.children.iter().map(expand_node)} }
            )
        }
        Node::Text(text) => rsx!(
            span { "{text.value}" }
        ),
        Node::Code(code) => {
            rsx!(
                pre {
                    code { language: if let Some(lang) = &code.lang { "{lang}" }, "{code.value}" }
                }
            )
        }
        Node::Heading(head) => {
            let children = head.children.iter().map(expand_node);
            match head.depth {
                1 => rsx!(
                    h1 { {children} }
                ),
                2 => rsx!(
                    h2 { {children} }
                ),
                3 => rsx!(
                    h3 { {children} }
                ),
                4 => rsx!(
                    h4 { {children} }
                ),
                5 => rsx!(
                    h5 { {children} }
                ),
                6 => rsx!(
                    h6 { {children} }
                ),
                _ => rsx!(
                    div { {children} }
                ),
            }
        }
        Node::Table(table) => rsx!(
            table { {table.children.iter().map(expand_node)} }
        ),
        Node::ThematicBreak(_) => rsx!(
            hr {}
        ),
        Node::TableRow(tr) => rsx!(
            tr { {tr.children.iter().map(expand_node)} }
        ),
        Node::TableCell(tc) => rsx!(
            td { {tc.children.iter().map(expand_node)} }
        ),
        Node::ListItem(li) => {
            if li.children.len() == 1
                && let Node::Paragraph(par) = &li.children[0] {
                    return rsx!(
                        li { style: if li.checked.is_some() { "display: flex" },
                            if let Some(checked) = li.checked {
                                input {
                                    r#type: "checkbox",
                                    style: "pointer-events: none; margin-right: 0.5em;",
                                    checked,
                                }
                            }
                            {par.children.iter().map(expand_node)}
                        }
                    );
                }
            rsx!(
                li { style: if li.checked.is_some() { "display: flex" },
                    if let Some(checked) = li.checked {
                        input {
                            r#type: "checkbox",
                            style: "pointer-events: none; margin-right: 0.5em;",
                            checked,
                        }
                    }
                    {li.children.iter().map(expand_node)}
                }
            )
        }
        Node::Definition(def) => {
            rsx!(
                a {
                    href: "{def.url}",
                    title: if let Some(title) = &def.title { "{title}" },
                    "{def.identifier}"
                }
            )
        }
        Node::Paragraph(par) => rsx!(
            p { {par.children.iter().map(expand_node)} }
        ),
        _ => {
            error!("Not implemented!");
            rsx! {
                p { "Not implemented!" }
            }
        }
    }
}
