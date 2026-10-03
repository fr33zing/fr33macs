use std::{
    collections::{HashMap, VecDeque},
    ops::Range,
};

use anyhow::{Context, Result};

use crate::input::Input;

pub struct ParseResult {
    pub spans: Vec<Range<usize>>,
    pub comments: Vec<Range<usize>>,
}

impl ParseResult {
    pub fn apply_offset(&mut self, offset: usize) {
        for span in self.spans.iter_mut() {
            span.start += offset;
            span.end += offset;
        }

        for range in self.comments.iter_mut() {
            range.start += offset;
            range.end += offset;
        }
    }

    pub fn max_depth(&self) -> usize {
        self.spans.len()
    }
}

pub fn parse(input: &Input) -> Result<ParseResult> {
    const QUOTE: char = '"';
    const ESCAPE: char = '\\';
    const COMMENT: char = ';';
    const NEWLINE: char = '\n';
    const PAREN_OPEN: char = '(';
    const PAREN_CLOSE: char = ')';

    let Input {
        point,
        offset,
        contents,
    } = input;

    let point = point - offset;
    let mut point_found = false;
    let mut point_parent_stack: VecDeque<usize> = VecDeque::new();
    let mut spans: HashMap<usize, Option<(usize, usize)>> = HashMap::new();
    let mut comments = Vec::<Range<usize>>::new();
    let mut depth: usize = 0;

    'iter_chars: {
        enum Mode {
            Normal,
            Quoted(bool),
            Commented(usize),
        }
        let mut mode = Mode::Normal;
        let mut parent_stack: VecDeque<usize> = VecDeque::new();

        for (i, c) in contents.chars().enumerate() {
            if i == point {
                point_found = true;
                point_parent_stack = parent_stack.clone();
            }

            let end = i + 1;
            mode = match mode {
                Mode::Normal => match c {
                    QUOTE => Mode::Quoted(false),
                    COMMENT => Mode::Commented(i),
                    PAREN_OPEN => {
                        parent_stack.push_back(i);
                        spans.insert(i, None);
                        depth += 1;
                        mode
                    }
                    PAREN_CLOSE => {
                        depth = depth.checked_sub(1).context("unmatched closing paren")?;
                        let start = parent_stack.pop_back().context("parent stack is empty")?;
                        spans.insert(start, Some((end, depth)));
                        if depth == 0 && point_found {
                            break 'iter_chars;
                        }
                        mode
                    }
                    _ => mode,
                },
                Mode::Quoted(escaped) if escaped => Mode::Quoted(false),
                Mode::Quoted(_) => match c {
                    QUOTE => Mode::Normal,
                    ESCAPE => Mode::Quoted(true),
                    _ => mode,
                },
                Mode::Commented(start) => match c {
                    NEWLINE => {
                        comments.push(start..end);
                        Mode::Normal
                    }
                    _ => mode,
                },
            };
        }
    }

    let mut spans = spans
        .into_iter()
        .filter_map(|(start, end)| {
            if let Some((end, _)) = end {
                if point_parent_stack.contains(&start) {
                    Some(start..end)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    spans.sort_unstable_by_key(|span| span.start);

    Ok(ParseResult { spans, comments })
}
