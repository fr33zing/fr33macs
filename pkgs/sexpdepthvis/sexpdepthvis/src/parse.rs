use std::{
    collections::{HashMap, VecDeque},
    ops::Range,
};

use anyhow::{Context, Result};

use crate::input::Input;

pub struct Span {
    pub range: Range<usize>,
    pub depth: usize,
}

pub struct ParseResult {
    pub spans: Vec<Span>,
    pub quoted_ranges: Vec<Range<usize>>,
    pub commented_ranges: Vec<Range<usize>>,
}

impl ParseResult {
    pub fn apply_offset(&mut self, offset: usize) {
        for span in self.spans.iter_mut() {
            span.range.start += offset;
            span.range.end += offset;
        }

        for range in self.quoted_ranges.iter_mut() {
            range.start += offset;
            range.end += offset;
        }

        for range in self.commented_ranges.iter_mut() {
            range.start += offset;
            range.end += offset;
        }
    }

    pub fn max_depth(&self) -> usize {
        if let Some(max) = self.spans.iter().max_by_key(|span| span.depth) {
            max.depth
        } else {
            0
        }
    }
}

pub fn parse(input: &Input) -> Result<ParseResult> {
    let Input {
        point,
        offset,
        contents,
    } = input;
    let point = point - offset;
    let mut spans: HashMap<usize, Option<(usize, usize)>> = HashMap::new();
    let mut parent_stack: VecDeque<usize> = VecDeque::new();
    let mut save_parent_stack: VecDeque<usize> = VecDeque::new();
    let mut found_point = false;
    let mut commented_ranges = Vec::<Range<usize>>::new();
    let mut comment_start = None::<usize>;
    let mut quoted_ranges = Vec::<Range<usize>>::new();
    let mut quote_start = None::<usize>;

    let mut depth = 0;

    for (i, c) in contents.chars().enumerate() {
        if i == point {
            found_point = true;
            save_parent_stack = parent_stack.clone();
        }

        if let Some(comment_start_inner) = comment_start {
            if c == '\n' {
                commented_ranges.push(comment_start_inner..i + 1);
                comment_start = None;
            }
            continue;
        } else if c == ';' {
            comment_start = Some(i);
            continue;
        }

        if let Some(quote_start_inner) = quote_start {
            if c == '"' {
                quoted_ranges.push(quote_start_inner..i + 1);
                quote_start = None;
            }

            continue;
        } else if c == '"' {
            quote_start = Some(i);
            continue;
        }

        if c == '(' {
            parent_stack.push_back(i);
            spans.insert(i, None);
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            let open = parent_stack.pop_back().unwrap();
            spans.insert(open, Some((i, depth)));
            if depth == 0 && found_point {
                break;
            }
        }
    }

    let mut spans = spans
        .into_iter()
        .filter_map(|(start, end)| {
            if let Some((end, depth)) = end {
                if save_parent_stack.contains(&start) {
                    Some(Span {
                        range: start..end,
                        depth,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    spans.sort_by_key(|x| x.range.start);

    let min_depth = spans.first().context(format!("spans is empty"))?.depth;
    spans.iter_mut().for_each(|span| span.depth -= min_depth);

    Ok(ParseResult {
        spans,
        quoted_ranges,
        commented_ranges,
    })
}
