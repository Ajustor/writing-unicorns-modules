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
                if chars[i] == '-' && chars[i+1] == '-' && chars[i+2] == '>' {
                    i += 3;
                    break;
                }
                i += 1;
            }
            if i >= len { i = len; }
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Comment });
            continue;
        }

        // Processing instruction: <?xml ... ?>
        if i + 1 < len && chars[i] == '<' && chars[i+1] == '?' {
            let start = i;
            i += 2;
            while i + 1 < len {
                if chars[i] == '?' && chars[i+1] == '>' { i += 2; break; }
                i += 1;
            }
            if i >= len { i = len; }
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Macro });
            continue;
        }

        // CDATA: <![CDATA[ ... ]]>
        if i + 8 < len && chars[i..i+9].iter().collect::<String>() == "<![CDATA[" {
            let start = i;
            i += 9;
            while i + 2 < len {
                if chars[i] == ']' && chars[i+1] == ']' && chars[i+2] == '>' { i += 3; break; }
                i += 1;
            }
            if i >= len { i = len; }
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Macro });
            continue;
        }

        // Tag: <tagname or </tagname or />  or >
        if chars[i] == '<' {
            let mut s = String::new();
            s.push('<');
            i += 1;
            if i < len && chars[i] == '/' { s.push('/'); i += 1; }
            if i < len && chars[i] == '!' { // DOCTYPE
                while i < len && chars[i] != '>' { s.push(chars[i]); i += 1; }
                if i < len { s.push('>'); i += 1; }
                tokens.push(Token { text: s, kind: TokenKind::Macro });
                continue;
            }
            // Tag name
            while i < len && chars[i].is_alphanumeric() || i < len && (chars[i] == ':' || chars[i] == '-' || chars[i] == '_' || chars[i] == '.') {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(Token { text: s, kind: TokenKind::Keyword });

            // Attributes inside tag
            while i < len && chars[i] != '>' {
                // Whitespace
                if chars[i].is_whitespace() {
                    let ws_start = i;
                    while i < len && chars[i].is_whitespace() { i += 1; }
                    tokens.push(Token { text: chars[ws_start..i].iter().collect(), kind: TokenKind::Normal });
                    continue;
                }
                // Self-closing />
                if chars[i] == '/' && i + 1 < len && chars[i+1] == '>' {
                    tokens.push(Token { text: "/>".to_string(), kind: TokenKind::Keyword });
                    i += 2;
                    break;
                }
                // Attribute name
                if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == ':' {
                    let attr_start = i;
                    while i < len && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-' || chars[i] == ':' || chars[i] == '.') {
                        i += 1;
                    }
                    tokens.push(Token { text: chars[attr_start..i].iter().collect(), kind: TokenKind::Property });
                    continue;
                }
                // = sign
                if chars[i] == '=' {
                    tokens.push(Token { text: "=".to_string(), kind: TokenKind::Normal });
                    i += 1;
                    continue;
                }
                // Attribute value string
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
                // Other char
                tokens.push(Token { text: chars[i].to_string(), kind: TokenKind::Normal });
                i += 1;
            }
            // Closing >
            if i < len && chars[i] == '>' {
                tokens.push(Token { text: ">".to_string(), kind: TokenKind::Keyword });
                i += 1;
            }
            continue;
        }

        // Entity: &amp; &#123;
        if chars[i] == '&' {
            let start = i;
            i += 1;
            while i < len && chars[i] != ';' && !chars[i].is_whitespace() { i += 1; }
            if i < len && chars[i] == ';' { i += 1; }
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Number });
            continue;
        }

        // Text content
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
pub extern "C" fn language_id() -> *const std::ffi::c_char { c"xml".as_ptr() }

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char { c"xml,xsl,xsd,svg,xhtml".as_ptr() }

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

pub fn hover_info(_word: &str, _file_content: &str) -> Option<String> { None }

#[no_mangle]
pub unsafe extern "C" fn hover_info_ffi(_w: *const std::ffi::c_char, _c: *const std::ffi::c_char) -> *mut std::ffi::c_char {
    std::ptr::null_mut()
}

pub fn lsp_server_command() -> Option<(String, Vec<String>)> {
    Some(("lemminx".to_string(), vec![]))
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
        assert!(tokens.iter().any(|t| t.kind == TokenKind::String));
    }

    #[test]
    fn test_processing_instruction() {
        let tokens = tokenize_line("<?xml version=\"1.0\"?>");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Macro));
    }

    #[test]
    fn test_entity() {
        let tokens = tokenize_line("&amp;");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Number));
    }

    #[test]
    fn test_self_closing() {
        let tokens = tokenize_line("<br/>");
        assert!(tokens.iter().any(|t| t.text == "/>" && t.kind == TokenKind::Keyword));
    }
}
