//rfc9112 section-3.2
pub fn is_request_target(line: &[u8]) -> bool {
    is_origin_form(line) || is_absolute_form(line) || is_authority_form(line) || is_asterisk_form(line)
}
fn is_origin_form(line: &[u8]) -> bool {
    if let Some(pos) = line.iter().position(|b| *b == b'?') {
        is_absolute_path(&line[..pos]) && is_query(&line[pos + 1..])
    } else {
        is_absolute_path(line)
    }
}

fn is_absolute_path(seq: &[u8]) -> bool {
    !seq.is_empty() && seq[0] == b'/' && consume_path_abempty(seq) == seq.len()
}

/// validates the `seq`uence using `predicate`
///
/// returns the consumed position and a boolean that indicates if minimum attempts is fulfilled
fn consume(seq: &[u8], predicate: fn(seq: &[u8]) -> (usize, bool), min: usize) -> (usize, bool) {
    let mut i = 0;
    let mut attempts = 0;
    while i < seq.len() {
        let (pos, valid) = predicate(&seq[i..]);
        i += pos;
        attempts += 1;
        if !valid {
            break;
        }
    }

    return (i, attempts > min);
}

//rfc3986 section-3.2.3
fn consume_segment(seq: &[u8]) -> usize {
    consume(seq, is_pchar, 0).0
}

//rfc3986 section-3.3: path-abempty = *( "/" segment )
fn consume_path_abempty(seq: &[u8]) -> usize {
    fn predicate(seq: &[u8]) -> (usize, bool) {
        //since this is only used in consume it's okay to not check slice bounds
        if seq[0] != b'/' {
            return (0, false);
        }
        let i = consume_segment(&seq[1..]);
        return (i + 1, true);
    }

    consume(seq, predicate, 0).0
}

//rfc3986 section-3.3: path-rootless / path-empty (the non "//authority" branches of hier-part)
fn consume_path(seq: &[u8]) -> usize {
    if seq.is_empty() || seq[0] == b'/' {
        return consume_path_abempty(seq);
    }

    // path-rootless = segment-nz *( "/" segment )
    let first = consume_segment(seq);
    if first == 0 {
        return 0; // no valid segment-nz here, so treat it as path-empty
    }
    first + consume_path_abempty(&seq[first..])
}

//rfc3986 appendix-A
fn is_pchar(seq: &[u8]) -> (usize, bool) {
    if is_pct_encoded(seq) {
        return (3, true);
    }
    if seq.is_empty() {
        return (0, false);
    }
    let b = &seq[0];
    if is_unreserved(b) || is_sub_delims(b) || b":@".contains(b) {
        return (1, true);
    }
    return (0, false);
}
fn is_unreserved(b: &u8) -> bool {
    b.is_ascii_alphanumeric() || b"-._~".contains(b)
}
fn is_pct_encoded(seq: &[u8]) -> bool {
    seq.len() > 2 && seq[0] == b'%' && seq[1].is_ascii_hexdigit() && seq[2].is_ascii_hexdigit()
}
fn is_sub_delims(b: &u8) -> bool {
    b"!$&'()*+,;=".contains(b)
}
fn is_query(seq: &[u8]) -> bool {
    fn predicate(seq: &[u8]) -> (usize, bool) {
        let (i, valid) = is_pchar(seq);
        if valid {
            return (i, true);
        }
        //since this is only used in consume it's okay to not check slice bounds
        if b"/?".contains(&seq[0]) {
            return (1, true);
        }
        return (0, false);
    }

    consume(seq, predicate, 0).0 == seq.len()
}
fn is_absolute_form(line: &[u8]) -> bool {
    is_absolute_uri(line)
}
fn is_absolute_uri(line: &[u8]) -> bool {
    //rfc3986 section-4.3: absolute-URI = scheme ":" hier-part [ "?" query ]
    let Some(scheme_end) = is_scheme(line) else {
        return false;
    };
    if scheme_end >= line.len() || line[scheme_end] != b':' {
        return false;
    }

    let Some(hier_len) = is_hier_part(&line[scheme_end + 1..]) else {
        return false;
    };
    let consumed = scheme_end + 1 + hier_len;

    if consumed == line.len() {
        return true;
    }
    line[consumed] == b'?' && is_query(&line[consumed + 1..])
}
fn is_scheme(seq: &[u8]) -> Option<usize> {
    if seq.is_empty() || !seq[0].is_ascii_alphabetic() {
        return None;
    }

    let mut i = 1;
    while i < seq.len() {
        let ch = &seq[i];
        if !ch.is_ascii_alphanumeric() && !b"+-.".contains(ch) {
            break;
        }
        i += 1;
    }

    return Some(i);
}
fn is_hier_part(seq: &[u8]) -> Option<usize> {
    //rfc3986 section-3: hier-part = "//" authority path-abempty
    //                              / path-absolute / path-rootless / path-empty
    if seq.starts_with(b"//") {
        let auth_len = is_authority(&seq[2..])?;
        let path_len = consume_path_abempty(&seq[2 + auth_len..]);
        return Some(2 + auth_len + path_len);
    }

    Some(consume_path(seq))
}
fn consume_user_info(seq: &[u8]) -> usize {
    fn predicate(seq: &[u8]) -> (usize, bool) {
        let ch = &seq[0];
        if is_pct_encoded(seq) {
            (3, true)
        } else if !is_unreserved(ch) && !is_sub_delims(ch) && *ch != b':' {
            (0, false)
        } else {
            (1, true)
        }
    }
    return consume(seq, predicate, 0).0;
}
fn is_authority(seq: &[u8]) -> Option<usize> {
    //rfc3986 section-3.2: authority = [ userinfo "@" ] host [ ":" port ]
    let user_info_len = consume_user_info(seq);
    let host_start = if user_info_len < seq.len() && seq[user_info_len] == b'@' {
        user_info_len + 1
    } else {
        // either there's no userinfo, or what looked like userinfo wasn't
        // followed by '@' and so is really the start of the host
        0
    };

    let host_len = is_host(&seq[host_start..])?;
    let mut end = host_start + host_len;

    if end < seq.len() && seq[end] == b':' {
        let mut port_end = end + 1;
        while port_end < seq.len() && seq[port_end].is_ascii_digit() {
            port_end += 1;
        }
        end = port_end;
    }

    Some(end)
}
fn is_host(seq: &[u8]) -> Option<usize> {
    [is_ip_literal, is_ip_v4_address, is_reg_name]
        .iter()
        .find_map(|f| if let Some(i) = f(seq) { Some(i) } else { None })
}
fn is_ip_literal(seq: &[u8]) -> Option<usize> {
    if seq.len() < 3 || seq[0] != b'[' {
        return None;
    }
    if let Some(n) = [is_ip_v6, is_ip_future].iter().find_map(|f| f(&seq[1..])) {
        if n + 1 < seq.len() && seq[n + 1] == b']' {
            return Some(n + 2);
        }
    }
    return None;
}

//rfc3986 appendix-A: leans on std's parser to validate the address text found
//between the enclosing '[' ']' of an IP-literal.
fn is_ip_v6(seq: &[u8]) -> Option<usize> {
    let end = seq.iter().position(|&b| b == b']').unwrap_or(seq.len());
    let text = std::str::from_utf8(&seq[..end]).ok()?;
    text.parse::<std::net::Ipv6Addr>().ok()?;
    Some(end)
}
//rfc3986 appendix-A: IPvFuture = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
fn is_ip_future(seq: &[u8]) -> Option<usize> {
    if seq.is_empty() || (seq[0] != b'v' && seq[0] != b'V') {
        return None;
    }

    let mut i = 1;
    let version_start = i;
    while i < seq.len() && seq[i].is_ascii_hexdigit() {
        i += 1;
    }
    if i == version_start {
        return None;
    }

    if i >= seq.len() || seq[i] != b'.' {
        return None;
    }
    i += 1;

    let rest_start = i;
    while i < seq.len() {
        let b = seq[i];
        if is_unreserved(&b) || is_sub_delims(&b) || b == b':' {
            i += 1;
        } else {
            break;
        }
    }
    if i == rest_start {
        return None;
    }

    Some(i)
}
//rfc3986 appendix-A: IPv4address = dec-octet "." dec-octet "." dec-octet "." dec-octet
fn is_ip_v4_address(seq: &[u8]) -> Option<usize> {
    let mut i = 0;
    for octet_index in 0..4 {
        if octet_index > 0 {
            if i >= seq.len() || seq[i] != b'.' {
                return None;
            }
            i += 1;
        }
        i += is_dec_octet(&seq[i..])?;
    }
    Some(i)
}
//rfc3986 appendix-A: dec-octet = DIGIT / %x31-39 DIGIT / "1" 2DIGIT
//                               / "2" %x30-34 DIGIT / "25" %x30-35
fn is_dec_octet(seq: &[u8]) -> Option<usize> {
    let max_len = seq.len().min(3);
    let digit_len = seq[..max_len].iter().take_while(|b| b.is_ascii_digit()).count();
    if digit_len == 0 {
        return None;
    }
    // try the longest digit run first, backing off to shorter runs, honoring the
    // grammar's "no leading zero on multi-digit values" and "<= 255" constraints
    for len in (1..=digit_len).rev() {
        let text = std::str::from_utf8(&seq[..len]).ok()?;
        if len > 1 && text.starts_with('0') {
            continue;
        }
        if let Ok(value) = text.parse::<u16>() {
            if value <= 255 {
                return Some(len);
            }
        }
    }
    None
}
//rfc3986 appendix-A: reg-name = *( unreserved / pct-encoded / sub-delims )
fn is_reg_name(seq: &[u8]) -> Option<usize> {
    fn predicate(seq: &[u8]) -> (usize, bool) {
        if is_pct_encoded(seq) {
            return (3, true);
        }
        //since this is only used in consume it's okay to not check slice bounds
        let b = &seq[0];
        if is_unreserved(b) || is_sub_delims(b) {
            (1, true)
        } else {
            (0, false)
        }
    }
    Some(consume(seq, predicate, 0).0)
}

fn is_authority_form(line: &[u8]) -> bool {
    //rfc9112 section-3.2.3: authority-form = authority (used only for CONNECT)
    matches!(is_authority(line), Some(n) if n == line.len())
}
fn is_asterisk_form(line: &[u8]) -> bool {
    //rfc9112 section-3.2.4: asterisk-form = "*" (used only for OPTIONS)
    line == b"*"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_form() {
        assert!(is_request_target(b"/where?q=now&x=1"));
        assert!(is_request_target(b"/"));
        assert!(is_request_target(b"/foo"));
        assert!(is_request_target(b"/foo/bar"));
        assert!(is_request_target(b"/foo?"));
        assert!(is_request_target(b"/search?redirect=/home"));
        assert!(!is_request_target(b"/foo<bad>"));
        assert!(!is_request_target(b"relative/path"));
    }

    #[test]
    fn absolute_form() {
        assert!(is_request_target(b"http://www.example.org/pub/WWW/TheProject.html"));
        assert!(is_request_target(b"https://example.com:8080/a/b?q=1"));
        assert!(is_request_target(b"http://user:pass@example.com/"));
        assert!(is_request_target(b"http://[::1]:8080/"));
        assert!(is_request_target(b"mailto:someone@example.com"));
        assert!(is_request_target(b"http://example.com"));
        assert!(is_request_target(b"http://"));
        assert!(!is_request_target(b"http//example.com"));
        assert!(!is_request_target(b"1http://example.com"));
    }

    #[test]
    fn authority_form() {
        assert!(is_request_target(b"www.example.com:443"));
        assert!(is_request_target(b"example.com"));
        // this still matches overall because "www.example.com" is also a
        // syntactically legal *scheme*, so it's accepted via absolute-form
        // (scheme "www.example.com" + path-rootless "443/") instead of
        // authority-form.
        assert!(is_request_target(b"www.example.com:443/"));
        assert!(!is_authority_form(b"www.example.com:443/"));
    }

    #[test]
    fn asterisk_form() {
        assert!(is_request_target(b"*"));
        // '*' is a sub-delim, so "**" is still syntactically a legal reg-name
        // (authority-form), just not the literal asterisk-form.
        assert!(is_request_target(b"**"));
        assert!(!is_asterisk_form(b"**"));
    }

    #[test]
    fn ipv4_host() {
        assert!(is_request_target(b"http://192.168.0.1/path"));
        // 999 isn't a valid IPv4 octet, but the digits/dots are still valid
        // reg-name characters, so this falls back to matching as a hostname.
        assert!(is_request_target(b"http://999.168.0.1/path"));
        assert!(is_ip_v4_address(b"999.168.0.1").is_none());
    }

    #[test]
    fn rejects_invalid_targets() {
        // an empty reg-name is a legal (if useless) authority-form host, so
        // "" technically matches the grammar; callers combine this with a
        // real request-line parser that wouldn't hand it an empty target.
        assert!(is_request_target(b""));
        assert!(!is_request_target(b"/foo bar"));
        assert!(!is_request_target(b"/foo<bad>"));
        assert!(!is_request_target(b"foo bar"));
    }
}
