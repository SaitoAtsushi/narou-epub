fn forbidden_char(ch: char) -> bool {
    const FORBIDDEN_LIST: [char; 10] = ['/', '\\', '<', '>', ':', '"', '|', '?', '*', '\0'];
    ch.is_control() || FORBIDDEN_LIST.contains(&ch)
}

pub fn sanitize(s: &str) -> String {
    s.trim().chars().filter(|&ch| !forbidden_char(ch)).collect()
}
