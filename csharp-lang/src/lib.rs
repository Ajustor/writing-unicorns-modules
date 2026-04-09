#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword,
    KeywordType,
    String,
    Comment,
    Number,
    Function,
    Macro,
    Normal,
    Property,
}

impl TokenKind {
    pub fn color_category(&self) -> &'static str {
        match self {
            TokenKind::Keyword => "keyword",
            TokenKind::KeywordType => "type",
            TokenKind::String => "string",
            TokenKind::Comment => "comment",
            TokenKind::Number => "number",
            TokenKind::Function => "function",
            TokenKind::Macro => "macro",
            TokenKind::Normal => "normal",
            TokenKind::Property => "property",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

const KEYWORDS: &[&str] = &[
    "abstract", "as", "base", "break", "case", "catch", "checked", "class",
    "const", "continue", "default", "delegate", "do", "else", "enum", "event",
    "explicit", "extern", "false", "finally", "fixed", "for", "foreach", "goto",
    "if", "implicit", "in", "interface", "internal", "is", "lock", "namespace",
    "new", "null", "operator", "out", "override", "params", "private", "protected",
    "public", "readonly", "ref", "return", "sealed", "sizeof", "stackalloc",
    "static", "struct", "switch", "this", "throw", "true", "try", "typeof",
    "unchecked", "unsafe", "using", "virtual", "void", "volatile", "while",
    "async", "await", "var", "dynamic", "yield", "when", "where", "nameof",
    "record", "init", "required", "with", "not", "and", "or",
];

const TYPE_KEYWORDS: &[&str] = &[
    "bool", "byte", "sbyte", "char", "decimal", "double", "float", "int", "uint",
    "long", "ulong", "short", "ushort", "string", "object", "nint", "nuint",
    "String", "Int32", "Int64", "Boolean", "Double", "Single", "Decimal",
    "Object", "Byte", "Char", "DateTime", "TimeSpan", "Guid",
    "List", "Dictionary", "IEnumerable", "IList", "IDictionary", "ICollection",
    "Task", "ValueTask", "Action", "Func", "Nullable", "Span", "Memory",
    "HashSet", "Queue", "Stack", "LinkedList", "SortedList",
];

pub fn tokenize_line(line: &str) -> Vec<Token> {
    let trimmed = line.trim_start();

    // Full-line comment
    if trimmed.starts_with("//") {
        return vec![Token { text: line.to_string(), kind: TokenKind::Comment }];
    }

    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Block comment start /* ... */
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < len {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            if i >= len { i = len; }
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Comment });
            continue;
        }

        // Inline comment //
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            tokens.push(Token { text: chars[i..].iter().collect(), kind: TokenKind::Comment });
            break;
        }

        // Verbatim string @"..."
        if chars[i] == '@' && i + 1 < len && chars[i + 1] == '"' {
            let mut s = String::new();
            s.push('@');
            s.push('"');
            i += 2;
            while i < len {
                if chars[i] == '"' {
                    if i + 1 < len && chars[i + 1] == '"' {
                        s.push('"');
                        s.push('"');
                        i += 2;
                    } else {
                        s.push('"');
                        i += 1;
                        break;
                    }
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            tokens.push(Token { text: s, kind: TokenKind::String });
            continue;
        }

        // Interpolated string $"..."
        if chars[i] == '$' && i + 1 < len && chars[i + 1] == '"' {
            let mut s = String::new();
            s.push('$');
            s.push('"');
            i += 2;
            while i < len {
                let c = chars[i];
                s.push(c);
                i += 1;
                if c == '\\' && i < len {
                    s.push(chars[i]);
                    i += 1;
                } else if c == '"' {
                    break;
                }
            }
            tokens.push(Token { text: s, kind: TokenKind::String });
            continue;
        }

        // Raw string literal $@"..." or @$"..."
        if i + 2 < len
            && ((chars[i] == '$' && chars[i + 1] == '@' && chars[i + 2] == '"')
                || (chars[i] == '@' && chars[i + 1] == '$' && chars[i + 2] == '"'))
        {
            let mut s: String = chars[i..i + 3].iter().collect();
            i += 3;
            while i < len {
                if chars[i] == '"' {
                    if i + 1 < len && chars[i + 1] == '"' {
                        s.push('"');
                        s.push('"');
                        i += 2;
                    } else {
                        s.push('"');
                        i += 1;
                        break;
                    }
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            tokens.push(Token { text: s, kind: TokenKind::String });
            continue;
        }

        // Regular string
        if chars[i] == '"' {
            let mut s = String::new();
            s.push('"');
            i += 1;
            while i < len {
                let c = chars[i];
                s.push(c);
                i += 1;
                if c == '\\' && i < len {
                    s.push(chars[i]);
                    i += 1;
                } else if c == '"' {
                    break;
                }
            }
            tokens.push(Token { text: s, kind: TokenKind::String });
            continue;
        }

        // Char literal
        if chars[i] == '\'' {
            let mut s = String::new();
            s.push('\'');
            i += 1;
            while i < len {
                let c = chars[i];
                s.push(c);
                i += 1;
                if c == '\\' && i < len {
                    s.push(chars[i]);
                    i += 1;
                } else if c == '\'' {
                    break;
                }
            }
            tokens.push(Token { text: s, kind: TokenKind::String });
            continue;
        }

        // Number
        if chars[i].is_ascii_digit()
            || (chars[i] == '.' && i + 1 < len && chars[i + 1].is_ascii_digit())
        {
            let mut s = String::new();
            // Hex: 0x...
            if chars[i] == '0' && i + 1 < len && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                s.push(chars[i]);
                s.push(chars[i + 1]);
                i += 2;
                while i < len && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                    s.push(chars[i]);
                    i += 1;
                }
            } else {
                while i < len
                    && (chars[i].is_ascii_alphanumeric()
                        || chars[i] == '.'
                        || chars[i] == '_')
                {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            // Suffix: f, d, m, L, UL, etc.
            if i < len && (chars[i] == 'f' || chars[i] == 'F' || chars[i] == 'd' || chars[i] == 'D' || chars[i] == 'm' || chars[i] == 'M' || chars[i] == 'L' || chars[i] == 'U' || chars[i] == 'u') {
                s.push(chars[i]);
                i += 1;
                if i < len && (chars[i] == 'L' || chars[i] == 'l') {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            tokens.push(Token { text: s, kind: TokenKind::Number });
            continue;
        }

        // Attribute [Attribute]
        if chars[i] == '[' && i + 1 < len && chars[i + 1].is_alphabetic() {
            let start = i;
            let mut j = i + 1;
            let mut found_close = false;
            while j < len {
                if chars[j] == ']' {
                    j += 1;
                    found_close = true;
                    break;
                }
                j += 1;
            }
            if found_close {
                let attr: String = chars[start..j].iter().collect();
                tokens.push(Token { text: attr, kind: TokenKind::Macro });
                i = j;
                continue;
            }
        }

        // Preprocessor directive #region, #if, #endif, etc.
        if chars[i] == '#' && (i == 0 || chars[..i].iter().all(|c| c.is_whitespace())) {
            tokens.push(Token { text: chars[i..].iter().collect(), kind: TokenKind::Macro });
            break;
        }

        // Word (identifier/keyword)
        if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == '@' {
            let start = i;
            // @ prefix for verbatim identifiers
            if chars[i] == '@' {
                i += 1;
            }
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let bare_word = word.trim_start_matches('@');
            let kind = if TYPE_KEYWORDS.contains(&bare_word) {
                TokenKind::KeywordType
            } else if KEYWORDS.contains(&bare_word) {
                TokenKind::Keyword
            } else if i < len && chars[i] == '(' {
                TokenKind::Function
            } else if i < len && chars[i] == '<' {
                // Generic type like List<T>
                TokenKind::KeywordType
            } else {
                TokenKind::Normal
            };
            tokens.push(Token { text: word, kind });
            continue;
        }

        // Punctuation / operators
        let start = i;
        while i < len
            && !chars[i].is_alphanumeric()
            && chars[i] != '_'
            && chars[i] != '"'
            && chars[i] != '\''
            && chars[i] != '/'
            && chars[i] != '#'
            && chars[i] != '@'
            && chars[i] != '$'
            && chars[i] != '['
            && chars[i] != '.'
        {
            i += 1;
        }
        if i > start {
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::Normal,
            });
        }
        if i == start {
            tokens.push(Token {
                text: chars[i].to_string(),
                kind: TokenKind::Normal,
            });
            i += 1;
        }
    }

    tokens
}

// ── FFI ──────────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn language_id() -> *const std::ffi::c_char {
    c"csharp".as_ptr()
}

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char {
    c"cs,csx".as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn tokenize_line_ffi(
    line_ptr: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
    let line = unsafe { std::ffi::CStr::from_ptr(line_ptr).to_str().unwrap_or("") };
    let tokens = tokenize_line(line);
    let json = tokens_to_json(&tokens);
    std::ffi::CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        unsafe { drop(std::ffi::CString::from_raw(ptr)) }
    }
}

pub fn hover_info(word: &str, file_content: &str) -> Option<String> {
    let patterns = [
        format!("class {} ", word),
        format!("class {}:", word),
        format!("class {}<", word),
        format!("interface {} ", word),
        format!("interface {}:", word),
        format!("interface {}<", word),
        format!("struct {} ", word),
        format!("struct {}:", word),
        format!("enum {} ", word),
        format!("record {} ", word),
        format!("record {}<", word),
        format!(" {}(", word),
        format!("void {}(", word),
        format!("async {}(", word),
        format!("Task {}(", word),
        format!("Task<{}", word),
        format!("static {}(", word),
    ];

    for line in file_content.lines() {
        let trimmed = line.trim();
        for pat in &patterns {
            if trimmed.contains(pat.as_str()) {
                let sig = trimmed
                    .trim_end_matches('{')
                    .trim_end_matches("=>")
                    .trim_end();
                return Some(format!("```csharp\n{sig}\n```"));
            }
        }
    }
    None
}

#[no_mangle]
pub unsafe extern "C" fn hover_info_ffi(
    word_ptr: *const std::ffi::c_char,
    file_content_ptr: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
    let word = unsafe { std::ffi::CStr::from_ptr(word_ptr).to_str().unwrap_or("") };
    let content = unsafe {
        std::ffi::CStr::from_ptr(file_content_ptr)
            .to_str()
            .unwrap_or("")
    };
    match hover_info(word, content) {
        Some(s) => std::ffi::CString::new(s)
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

pub fn lsp_server_command() -> Option<(String, Vec<String>)> {
    Some(("OmniSharp".to_string(), vec!["-lsp".to_string()]))
}

fn tokens_to_json(tokens: &[Token]) -> String {
    let parts: Vec<String> = tokens
        .iter()
        .map(|t| {
            format!(
                r#"{{"text":{},"kind":"{}"}}"#,
                serde_json_escape(&t.text),
                t.kind.color_category()
            )
        })
        .collect();
    format!("[{}]", parts.join(","))
}

fn serde_json_escape(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword() {
        let tokens = tokenize_line("public class MyClass {");
        assert!(tokens.iter().any(|t| t.text == "public" && t.kind == TokenKind::Keyword));
        assert!(tokens.iter().any(|t| t.text == "class" && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn test_type_keyword() {
        let tokens = tokenize_line("int x = 0;");
        assert!(tokens.iter().any(|t| t.text == "int" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize_line("// this is a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_block_comment() {
        let tokens = tokenize_line("/* block */ int x;");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Comment));
        assert!(tokens.iter().any(|t| t.text == "int" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_string() {
        let tokens = tokenize_line(r#"string s = "hello";"#);
        assert!(tokens.iter().any(|t| t.text == "\"hello\"" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_verbatim_string() {
        let tokens = tokenize_line(r#"string s = @"C:\path";"#);
        assert!(tokens
            .iter()
            .any(|t| t.kind == TokenKind::String && t.text.starts_with("@\"")));
    }

    #[test]
    fn test_interpolated_string() {
        let tokens = tokenize_line(r#"var s = $"Hello {name}";"#);
        assert!(tokens
            .iter()
            .any(|t| t.kind == TokenKind::String && t.text.starts_with("$\"")));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize_line("int x = 42;");
        assert!(tokens.iter().any(|t| t.text == "42" && t.kind == TokenKind::Number));
    }

    #[test]
    fn test_attribute() {
        let tokens = tokenize_line("[Serializable]");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Macro));
    }

    #[test]
    fn test_preprocessor() {
        let tokens = tokenize_line("#region MyRegion");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Macro));
    }

    #[test]
    fn test_function() {
        let tokens = tokenize_line("public void DoSomething() {");
        assert!(tokens
            .iter()
            .any(|t| t.text == "DoSomething" && t.kind == TokenKind::Function));
    }

    #[test]
    fn test_async_await() {
        let tokens = tokenize_line("await Task.Run(() => {});");
        assert!(tokens.iter().any(|t| t.text == "await" && t.kind == TokenKind::Keyword));
        assert!(tokens.iter().any(|t| t.text == "Task" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_char_literal() {
        let tokens = tokenize_line("char c = 'A';");
        assert!(tokens.iter().any(|t| t.text == "'A'" && t.kind == TokenKind::String));
    }
}
