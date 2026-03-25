pub(crate) fn strip_comments(input: &str) -> String {
    let mut stripped = String::with_capacity(input.len());
    let mut index = 0;
    let mut in_comment = false;
    let mut in_code_block = false;

    while index < input.len() {
        let remainder = &input[index..];

        if in_comment {
            if remainder.starts_with("~}") {
                in_comment = false;
                index += 2;
                continue;
            }

            let ch = remainder.chars().next().unwrap();
            if ch == '\n' || ch == '\r' {
                stripped.push(ch);
            }
            index += ch.len_utf8();
            continue;
        }

        if is_code_fence_start(remainder) {
            let line_end = remainder
                .find('\n')
                .map(|offset| offset + 1)
                .unwrap_or(remainder.len());
            stripped.push_str(&remainder[..line_end]);
            in_code_block = !in_code_block;
            index += line_end;
            continue;
        }

        if !in_code_block && remainder.starts_with("{~") {
            in_comment = true;
            index += 2;
            continue;
        }

        let ch = remainder.chars().next().unwrap();
        stripped.push(ch);
        index += ch.len_utf8();
    }

    stripped
}

pub(crate) fn is_code_fence_start(input: &str) -> bool {
    if !input.starts_with("```") {
        return false;
    }

    input == "```" || input.starts_with("```\n") || input.starts_with("```\r\n")
}
