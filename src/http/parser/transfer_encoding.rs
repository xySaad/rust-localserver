use crate::http::consume;

/// RFC 9110 §5.6.2
fn is_token(ch: u8) -> bool {
    b"!#$%&'*+-.^_`|~".contains(&ch) || ch.is_ascii_alphanumeric()
}

fn tchar_unit(seq: &[u8]) -> (usize, bool) {
    match seq.first() {
        Some(&b) if is_token(b) => (1, true),
        _ => (0, false),
    }
}

/// RFC 9110 §5.6.3
fn is_ows(ch: u8) -> bool {
    b"\t ".contains(&ch)
}

fn ows_unit(seq: &[u8]) -> (usize, bool) {
    match seq.first() {
        Some(&b) if is_ows(b) => (1, true),
        _ => (0, false),
    }
}

const DQUOTE: u8 = b'"';

fn is_obs_text(ch: u8) -> bool {
    ch >= 0x80
}

fn is_vchar(ch: u8) -> bool {
    ch.is_ascii_graphic()
}

fn is_qdtext(ch: u8) -> bool {
    is_ows(ch) || ch == 0x21 || (0x23..=0x5B).contains(&ch) || (0x5D..=0x7E).contains(&ch) || is_obs_text(ch)
}

fn is_quoted_pair_char(ch: u8) -> bool {
    is_ows(ch) || is_vchar(ch) || is_obs_text(ch)
}

fn quoted_string_body_unit(seq: &[u8]) -> (usize, bool) {
    match seq.first() {
        Some(&b) if is_qdtext(b) => (1, true),
        Some(&b'\\') => match seq.get(1) {
            Some(&c) if is_quoted_pair_char(c) => (2, true),
            _ => (0, false),
        },
        _ => (0, false),
    }
}

fn consume_quoted_string(seq: &[u8]) -> usize {
    if seq.first() != Some(&DQUOTE) {
        return 0;
    }
    let (body_len, _) = consume(&seq[1..], quoted_string_body_unit, 0);
    if seq.get(1 + body_len) != Some(&DQUOTE) {
        return 0; // unterminated
    }
    1 + body_len + 1
}

/// RFC 9110 §10.1.4
fn consume_transfer_parameter(seq: &[u8]) -> usize {
    let (name_len, _) = consume(seq, tchar_unit, 0);
    if name_len == 0 {
        return 0;
    }
    let mut pos = name_len;

    pos += consume(&seq[pos..], ows_unit, 0).0; // BWS == OWS

    if seq.get(pos) != Some(&b'=') {
        return 0;
    }
    pos += 1;
    pos += consume(&seq[pos..], ows_unit, 0).0;

    if seq.get(pos) == Some(&DQUOTE) {
        let n = consume_quoted_string(&seq[pos..]);
        if n == 0 {
            return 0;
        }
        pos + n
    } else {
        let (val_len, _) = consume(&seq[pos..], tchar_unit, 0);
        if val_len == 0 {
            return 0;
        }
        pos + val_len
    }
}
fn transfer_coding_repetition_unit(seq: &[u8]) -> (usize, bool) {
    let mut pos = consume(seq, ows_unit, 0).0;

    if seq.get(pos) != Some(&b';') {
        return (0, false);
    }
    pos += 1;
    pos += consume(&seq[pos..], ows_unit, 0).0;

    let param_len = consume_transfer_parameter(&seq[pos..]);
    if param_len == 0 {
        return (0, false);
    }
    (pos + param_len, true)
}

/// RFC 9110 §10.1.4
pub fn is_transfer_encoding(text: &[u8]) -> bool {
    if text.is_empty() {
        return false;
    }

    let (tok_len, _) = consume(text, tchar_unit, 0);
    if tok_len == 0 {
        return false;
    }

    let (rest_len, _) = consume(&text[tok_len..], transfer_coding_repetition_unit, 0);
    tok_len + rest_len == text.len()
}
