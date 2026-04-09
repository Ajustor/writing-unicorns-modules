#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind { Keyword, String, Comment, Number, Macro, Normal, Property }

impl TokenKind {
    pub fn color_category(&self) -> &'static str {
        match self {
            TokenKind::Keyword => "keyword", TokenKind::String => "string",
            TokenKind::Comment => "comment", TokenKind::Number => "number",
            TokenKind::Macro => "macro", TokenKind::Normal => "normal",
            TokenKind::Property => "property",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token { pub text: String, pub kind: TokenKind }

const HTML_TAG_DOCS: &[(&str, &str)] = &[
    ("html", "Root element of an HTML document"),
    ("head", "Container for metadata (title, scripts, styles)"),
    ("body", "Container for the visible page content"),
    ("div", "Generic block-level container"),
    ("span", "Generic inline container"),
    ("p", "Paragraph"),
    ("a", "Hyperlink — use href attribute for the URL"),
    ("img", "Image — use src and alt attributes"),
    ("input", "Input control (text, checkbox, radio, etc.)"),
    ("button", "Clickable button"),
    ("form", "Form container for user input"),
    ("table", "Table element"),
    ("tr", "Table row"),
    ("td", "Table data cell"),
    ("th", "Table header cell"),
    ("ul", "Unordered list"),
    ("ol", "Ordered list"),
    ("li", "List item"),
    ("h1", "Heading level 1 (largest)"),
    ("h2", "Heading level 2"),
    ("h3", "Heading level 3"),
    ("h4", "Heading level 4"),
    ("h5", "Heading level 5"),
    ("h6", "Heading level 6 (smallest)"),
    ("header", "Header section of a page or section"),
    ("footer", "Footer section"),
    ("nav", "Navigation links container"),
    ("main", "Main content of the document"),
    ("section", "Thematic section"),
    ("article", "Self-contained content (blog post, news article)"),
    ("aside", "Sidebar or tangential content"),
    ("script", "Embedded JavaScript"),
    ("style", "Embedded CSS styles"),
    ("link", "External resource link (stylesheets, icons)"),
    ("meta", "Document metadata (charset, viewport, etc.)"),
    ("title", "Document title (shown in browser tab)"),
    ("br", "Line break (void element)"),
    ("hr", "Horizontal rule / thematic break"),
    ("label", "Label for a form control"),
    ("select", "Drop-down select list"),
    ("option", "Option in a select list"),
    ("textarea", "Multi-line text input"),
    ("video", "Video player"),
    ("audio", "Audio player"),
    ("canvas", "Drawing surface for JavaScript graphics"),
    ("iframe", "Inline frame for embedding another document"),
];

pub fn tokenize_line(line: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Comment: <!-- ... -->
        if i + 3 < len && chars[i] == '<' && chars[i+1] == '!' && chars[i+2] == '-' && chars[i+3] == '-' {
            let start = i;
            i += 4;
            while i + 2 < len {
                if chars[i] == '-' && chars[i+1] == '-' && chars[i+2] == '>' { i += 3; break; }
                i += 1;
            }
            if i >= len { i = len; }
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Comment });
            continue;
        }

        // DOCTYPE
        if i + 8 < len && chars[i..i+9].iter().collect::<String>().to_uppercase() == "<!DOCTYPE" {
            let start = i;
            while i < len && chars[i] != '>' { i += 1; }
            if i < len { i += 1; }
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Macro });
            continue;
        }

        // Tag
        if chars[i] == '<' {
            let mut s = String::new();
            s.push('<');
            i += 1;
            if i < len && chars[i] == '/' { s.push('/'); i += 1; }
            // Tag name
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_' || chars[i] == ':') {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(Token { text: s, kind: TokenKind::Keyword });

            // Attributes inside tag
            while i < len && chars[i] != '>' {
                if chars[i].is_whitespace() {
                    let ws_start = i;
                    while i < len && chars[i].is_whitespace() { i += 1; }
                    tokens.push(Token { text: chars[ws_start..i].iter().collect(), kind: TokenKind::Normal });
                    continue;
                }
                if chars[i] == '/' && i + 1 < len && chars[i+1] == '>' {
                    tokens.push(Token { text: "/>".to_string(), kind: TokenKind::Keyword });
                    i += 2;
                    break;
                }
                if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == ':' || chars[i] == '@' || chars[i] == 'v' {
                    let attr_start = i;
                    while i < len && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-' || chars[i] == ':' || chars[i] == '.' || chars[i] == '@') {
                        i += 1;
                    }
                    tokens.push(Token { text: chars[attr_start..i].iter().collect(), kind: TokenKind::Property });
                    continue;
                }
                if chars[i] == '=' {
                    tokens.push(Token { text: "=".to_string(), kind: TokenKind::Normal });
                    i += 1;
                    continue;
                }
                if chars[i] == '"' || chars[i] == '\'' {
                    let quote = chars[i];
                    let mut sv = String::new();
                    sv.push(quote);
                    i += 1;
                    while i < len && chars[i] != quote { sv.push(chars[i]); i += 1; }
                    if i < len { sv.push(chars[i]); i += 1; }
                    tokens.push(Token { text: sv, kind: TokenKind::String });
                    continue;
                }
                tokens.push(Token { text: chars[i].to_string(), kind: TokenKind::Normal });
                i += 1;
            }
            if i < len && chars[i] == '>' {
                tokens.push(Token { text: ">".to_string(), kind: TokenKind::Keyword });
                i += 1;
            }
            continue;
        }

        // Entity
        if chars[i] == '&' {
            let start = i;
            i += 1;
            while i < len && chars[i] != ';' && !chars[i].is_whitespace() { i += 1; }
            if i < len && chars[i] == ';' { i += 1; }
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Number });
            continue;
        }

        // Text
        let start = i;
        while i < len && chars[i] != '<' && chars[i] != '&' { i += 1; }
        if i > start {
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Normal });
        }
    }

    if tokens.is_empty() && !line.is_empty() {
        tokens.push(Token { text: line.to_string(), kind: TokenKind::Normal });
    }
    tokens
}

// ── FFI ──────────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn language_id() -> *const std::ffi::c_char { c"html".as_ptr() }

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char { c"html,htm".as_ptr() }

#[no_mangle]
pub unsafe extern "C" fn tokenize_line_ffi(line_ptr: *const std::ffi::c_char) -> *mut std::ffi::c_char {
    let line = unsafe { std::ffi::CStr::from_ptr(line_ptr).to_str().unwrap_or("") };
    let tokens = tokenize_line(line);
    let json = tokens_to_json(&tokens);
    std::ffi::CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { unsafe { drop(std::ffi::CString::from_raw(ptr)) } }
}

pub fn hover_info(word: &str, _file_content: &str) -> Option<String> {
    HTML_TAG_DOCS.iter()
        .find(|(tag, _)| *tag == word)
        .map(|(tag, desc)| format!("```html\n<{}>\n```\n{}", tag, desc))
}

#[no_mangle]
pub unsafe extern "C" fn hover_info_ffi(word_ptr: *const std::ffi::c_char, content_ptr: *const std::ffi::c_char) -> *mut std::ffi::c_char {
    let word = unsafe { std::ffi::CStr::from_ptr(word_ptr).to_str().unwrap_or("") };
    let content = unsafe { std::ffi::CStr::from_ptr(content_ptr).to_str().unwrap_or("") };
    match hover_info(word, content) {
        Some(s) => std::ffi::CString::new(s).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

pub fn lsp_server_command() -> Option<(String, Vec<String>)> {
    Some(("vscode-html-language-server".to_string(), vec!["--stdio".to_string()]))
}

fn tokens_to_json(tokens: &[Token]) -> String {
    let parts: Vec<String> = tokens.iter().map(|t| {
        format!(r#"{{"text":{},"kind":"{}"}}"#, serde_json_escape(&t.text), t.kind.color_category())
    }).collect();
    format!("[{}]", parts.join(","))
}

fn serde_json_escape(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment() {
        let tokens = tokenize_line("<!-- comment -->");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Comment));
    }

    #[test]
    fn test_tag() {
        let tokens = tokenize_line("<div>");
        assert!(tokens.iter().any(|t| t.text.contains("div") && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn test_attribute() {
        let tokens = tokenize_line(r#"<div class="main">"#);
        assert!(tokens.iter().any(|t| t.text == "class" && t.kind == TokenKind::Property));
    }

    #[test]
    fn test_doctype() {
        let tokens = tokenize_line("<!DOCTYPE html>");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Macro));
    }

    #[test]
    fn test_entity() {
        let tokens = tokenize_line("&nbsp;");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Number));
    }

    #[test]
    fn test_self_closing() {
        let tokens = tokenize_line("<br/>");
        assert!(tokens.iter().any(|t| t.text == "/>" && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn test_hover_div() {
        assert!(hover_info("div", "").is_some());
    }

    #[test]
    fn test_hover_unknown() {
        assert!(hover_info("zzz", "").is_none());
    }
}
