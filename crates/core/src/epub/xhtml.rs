/// Extrae párrafos de texto limpio desde un documento XHTML/HTML de un capítulo EPUB.
/// Descarta etiquetas de metadatos/scripts/estilos y decodifica entidades HTML comunes.
pub fn extract_paragraphs(xhtml: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut skip_depth: usize = 0;
    let mut skip_tag_name = String::new();

    for ch in xhtml.chars() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
                let tag_raw = tag_buf.trim();
                let is_closing = tag_raw.starts_with('/');
                let tag_name = tag_raw
                    .trim_start_matches('/')
                    .split(|c: char| c.is_whitespace() || c == '/')
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();

                if skip_depth > 0 {
                    if is_closing && tag_name == skip_tag_name {
                        skip_depth -= 1;
                        if skip_depth == 0 {
                            skip_tag_name.clear();
                        }
                    } else if !is_closing && !tag_raw.ends_with('/') && tag_name == skip_tag_name {
                        skip_depth += 1;
                    }
                } else if !is_closing
                    && !tag_raw.ends_with('/')
                    && matches!(
                        tag_name.as_str(),
                        "head" | "script" | "style" | "svg" | "noscript"
                    )
                {
                    skip_depth = 1;
                    skip_tag_name = tag_name;
                } else if is_block_boundary(&tag_name, is_closing) {
                    push_normalized(&mut paragraphs, &mut current);
                } else if tag_name == "br" {
                    if !current.ends_with('\n') && !current.is_empty() {
                        current.push(' ');
                    }
                }

                tag_buf.clear();
            } else {
                tag_buf.push(ch);
            }
        } else if ch == '<' {
            in_tag = true;
            tag_buf.clear();
        } else if skip_depth == 0 {
            current.push(ch);
        }
    }

    push_normalized(&mut paragraphs, &mut current);
    paragraphs
}

fn is_block_boundary(tag_name: &str, is_closing: bool) -> bool {
    let is_block = matches!(
        tag_name,
        "p" | "div"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "blockquote"
            | "li"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "tr"
            | "hr"
    );
    is_block && (is_closing || tag_name == "hr")
}

fn push_normalized(paragraphs: &mut Vec<String>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    let decoded = decode_html_entities(current);
    let normalized = collapse_whitespace(&decoded);
    if !normalized.is_empty() {
        paragraphs.push(normalized);
    }
    current.clear();
}

fn collapse_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_ws = true;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

fn decode_html_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            let mut closed = false;
            for _ in 0..12 {
                match chars.peek().copied() {
                    Some(';') => {
                        chars.next();
                        closed = true;
                        break;
                    }
                    Some(c) if c.is_ascii_alphanumeric() || c == '#' => {
                        entity.push(c);
                        chars.next();
                    }
                    _ => break,
                }
            }

            if closed {
                if let Some(decoded) = resolve_entity(&entity) {
                    out.push_str(&decoded);
                    continue;
                }
            }
            out.push('&');
            out.push_str(&entity);
            if closed {
                out.push(';');
            }
        } else {
            out.push(ch);
        }
    }

    out
}

fn resolve_entity(entity: &str) -> Option<String> {
    let named = match entity {
        "amp" => Some("&"),
        "lt" => Some("<"),
        "gt" => Some(">"),
        "quot" => Some("\""),
        "apos" | "#39" => Some("'"),
        "nbsp" => Some(" "),
        "mdash" => Some("—"),
        "ndash" => Some("–"),
        "hellip" => Some("…"),
        "laquo" => Some("«"),
        "raquo" => Some("»"),
        "lsquo" => Some("‘"),
        "rsquo" => Some("’"),
        "ldquo" => Some("“"),
        "rdquo" => Some("”"),
        _ => None,
    };
    if let Some(s) = named {
        return Some(s.to_string());
    }

    if let Some(hex) = entity.strip_prefix("#x").or_else(|| entity.strip_prefix("#X")) {
        if let Ok(code) = u32::from_str_radix(hex, 16) {
            if let Some(c) = char::from_u32(code) {
                return Some(c.to_string());
            }
        }
    } else if let Some(dec) = entity.strip_prefix('#') {
        if let Ok(code) = dec.parse::<u32>() {
            if let Some(c) = char::from_u32(code) {
                return Some(c.to_string());
            }
        }
    }

    None
}
