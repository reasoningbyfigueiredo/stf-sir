use anyhow::{bail, Result};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::compiler::lexical;
use crate::model::{
    Diagnostic, DiagnosticSeverity, LexicalDimension, LogicalDimension, SemanticDimension,
    SyntacticDimension, ZToken,
};

#[derive(Debug, Clone)]
pub struct SyntacticOutput {
    pub ztokens: Vec<ZToken>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct RawNode {
    node_type: &'static str,
    children: Vec<usize>,
    start: usize,
    end: usize,
}

/// Parser options used by STF-SIR v1. These are captured in `config_hash`
/// so any change here produces a distinct compiler configuration identity.
pub(crate) fn parser_options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES | Options::ENABLE_STRIKETHROUGH
}

pub fn parse_markdown(document: &lexical::LexicalDocument) -> Result<SyntacticOutput> {
    let parser = Parser::new_ext(&document.source, parser_options()).into_offset_iter();

    let mut nodes: Vec<RawNode> = Vec::new();
    let mut root_indexes: Vec<usize> = Vec::new();
    let mut open_nodes: Vec<usize> = Vec::new();
    let mut diagnostics = Vec::new();

    for (event, range) in parser {
        update_open_node_ends(&mut nodes, &open_nodes, range.end);

        match event {
            Event::Start(tag) => {
                if let Some(node_type) = supported_block_start(&tag) {
                    let parent = open_nodes.last().copied();
                    let index = nodes.len();
                    nodes.push(RawNode {
                        node_type,
                        children: Vec::new(),
                        start: range.start,
                        end: range.end,
                    });

                    if let Some(parent_index) = parent {
                        nodes[parent_index].children.push(index);
                    } else {
                        root_indexes.push(index);
                    }

                    open_nodes.push(index);
                } else if let Some(tag_name) = unsupported_block_start(&tag) {
                    diagnostics.push(Diagnostic {
                        code: "SYN_NODE_UNSUPPORTED".to_string(),
                        severity: DiagnosticSeverity::Warning,
                        message: format!("unsupported block node ignored: {tag_name}"),
                        token_id: None,
                        stage: "syntactic".to_string(),
                    });
                }
            }
            Event::End(tag_end) => {
                if let Some(expected_type) = supported_block_end(&tag_end) {
                    let Some(node_index) = open_nodes.pop() else {
                        bail!("unexpected markdown end event for supported block node");
                    };

                    let actual_type = nodes[node_index].node_type;
                    if actual_type != expected_type {
                        bail!(
                            "mismatched markdown block close: expected {actual_type}, got {expected_type}"
                        );
                    }
                }
            }
            _ => {}
        }
    }

    if !open_nodes.is_empty() {
        bail!("unterminated markdown block nodes in parser output");
    }

    let mut ctx = BuildCtx {
        nodes: &nodes,
        document,
        next_token_id: 1,
        ztokens: Vec::new(),
    };

    for (sibling_index, root_index) in root_indexes.iter().enumerate() {
        let frame = NodeFrame {
            node_index: *root_index,
            parent_id: None,
            depth: 0,
            sibling_index,
            path: sibling_index.to_string(),
        };
        ctx.build_ztokens(frame)?;
    }

    Ok(SyntacticOutput {
        ztokens: ctx.ztokens,
        diagnostics,
    })
}

struct BuildCtx<'a> {
    nodes: &'a [RawNode],
    document: &'a lexical::LexicalDocument,
    next_token_id: usize,
    ztokens: Vec<ZToken>,
}

struct NodeFrame {
    node_index: usize,
    parent_id: Option<String>,
    depth: usize,
    sibling_index: usize,
    path: String,
}

impl<'a> BuildCtx<'a> {
    fn build_ztokens(&mut self, frame: NodeFrame) -> Result<()> {
        let node = &self.nodes[frame.node_index];
        let token_id = format!("z{}", self.next_token_id);
        self.next_token_id += 1;

        let end = trim_trailing_line_endings(&self.document.source, node.start, node.end);
        let source_text = lexical::slice_source(&self.document.source, node.start, end)?;
        let plain_text = extract_plain_text(&source_text);
        let span = lexical::span_from_offsets(self.document, node.start, end);

        self.ztokens.push(ZToken {
            id: token_id.clone(),
            lexical: LexicalDimension {
                source_text,
                plain_text,
                normalized_text: String::new(),
                span,
            },
            syntactic: SyntacticDimension {
                node_type: node.node_type.to_string(),
                parent_id: frame.parent_id.clone(),
                depth: frame.depth,
                sibling_index: frame.sibling_index,
                path: frame.path.clone(),
            },
            semantic: SemanticDimension {
                gloss: String::new(),
                concepts: Vec::new(),
                confidence: None,
            },
            logical: LogicalDimension {
                relation_ids: Vec::new(),
            },
            extensions: std::collections::BTreeMap::new(),
        });

        let child_indexes: Vec<usize> = self.nodes[frame.node_index].children.clone();
        for (child_sibling_index, child_index) in child_indexes.into_iter().enumerate() {
            let child_frame = NodeFrame {
                node_index: child_index,
                parent_id: Some(token_id.clone()),
                depth: frame.depth + 1,
                sibling_index: child_sibling_index,
                path: format!("{}/{}", frame.path, child_sibling_index),
            };
            self.build_ztokens(child_frame)?;
        }

        Ok(())
    }
}

fn update_open_node_ends(nodes: &mut [RawNode], open_nodes: &[usize], new_end: usize) {
    for node_index in open_nodes {
        if nodes[*node_index].end < new_end {
            nodes[*node_index].end = new_end;
        }
    }
}

fn trim_trailing_line_endings(source: &str, start: usize, mut end: usize) -> usize {
    let bytes = source.as_bytes();
    while end > start && matches!(bytes[end - 1], b'\n' | b'\r') {
        end -= 1;
    }
    end
}

fn extract_plain_text(source_text: &str) -> String {
    let parser = Parser::new_ext(source_text, Options::empty());
    let mut plain_text = String::new();

    for event in parser {
        match event {
            Event::Text(text) | Event::Code(text) => plain_text.push_str(&text),
            Event::SoftBreak | Event::HardBreak => push_newline(&mut plain_text),
            Event::End(tag_end) if ends_block_with_separator(&tag_end) => {
                push_newline(&mut plain_text)
            }
            _ => {}
        }
    }

    plain_text.trim_end_matches('\n').to_string()
}

fn push_newline(buffer: &mut String) {
    if !buffer.ends_with('\n') && !buffer.is_empty() {
        buffer.push('\n');
    }
}

fn supported_block_start(tag: &Tag<'_>) -> Option<&'static str> {
    match tag {
        Tag::Heading { .. } => Some("heading"),
        Tag::Paragraph => Some("paragraph"),
        Tag::BlockQuote(_) => Some("blockquote"),
        Tag::List(_) => Some("list"),
        Tag::Item => Some("list_item"),
        Tag::CodeBlock(_) => Some("code_block"),
        Tag::Table(_) => Some("table"),
        Tag::FootnoteDefinition(_) => Some("footnote_definition"),
        _ => None,
    }
}

fn unsupported_block_start(tag: &Tag<'_>) -> Option<&'static str> {
    match tag {
        Tag::HtmlBlock => Some("html_block"),
        _ => None,
    }
}

fn supported_block_end(tag_end: &TagEnd) -> Option<&'static str> {
    match tag_end {
        TagEnd::Heading(_) => Some("heading"),
        TagEnd::Paragraph => Some("paragraph"),
        TagEnd::BlockQuote(_) => Some("blockquote"),
        TagEnd::List(_) => Some("list"),
        TagEnd::Item => Some("list_item"),
        TagEnd::CodeBlock => Some("code_block"),
        TagEnd::Table => Some("table"),
        TagEnd::FootnoteDefinition => Some("footnote_definition"),
        _ => None,
    }
}

fn ends_block_with_separator(tag_end: &TagEnd) -> bool {
    matches!(
        tag_end,
        TagEnd::Heading(_)
            | TagEnd::Paragraph
            | TagEnd::BlockQuote(_)
            | TagEnd::List(_)
            | TagEnd::Item
            | TagEnd::CodeBlock
            | TagEnd::Table
            | TagEnd::TableRow
            | TagEnd::FootnoteDefinition
    )
}
