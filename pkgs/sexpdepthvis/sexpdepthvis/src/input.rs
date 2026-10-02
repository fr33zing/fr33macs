use std::{
    collections::VecDeque,
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
};

use anyhow::{anyhow, bail, Error, Result};
use iterate_text::string::lines::IterateStringLines;

use crate::color::*;

const BEGIN_SRC_BLOCK: &str = "#+begin_src";
const END_SRC_BLOCK: &str = "#+end_src";

pub struct OffsetContents {
    pub offset: usize,
    pub contents: String,
}

pub struct Input {
    pub point: usize,
    pub offset: usize,
    pub contents: String,
}

impl Input {
    pub(crate) fn new(
        point: usize,
        offset: usize,
        major_mode: &str,
        contents: String,
    ) -> Result<Self> {
        let OffsetContents { offset, contents } = if major_mode != "org-mode" {
            OffsetContents { offset, contents }
        } else {
            println!("{}", 123123);
            println!("{}", contents);
            Self::org_src_block_contents_from_string(point, offset, contents)?
        };

        Ok(Self {
            point,
            offset,
            contents,
        })
    }

    pub fn new_from_args() -> Result<Self> {
        let mut args = env::args().skip(1).collect::<VecDeque<_>>();
        let point = args.pop_front().ok_or_else(Self::args_error)?.parse()?;
        let major_mode = args.pop_front().ok_or_else(Self::args_error)?;
        let file_path = args.pop_front().ok_or_else(Self::args_error)?;
        let OffsetContents { offset, contents } =
            Self::read_contents(point, &major_mode, &file_path)?;

        Ok(Self {
            point,
            offset,
            contents,
        })
    }

    fn args_error() -> Error {
        anyhow!("Invalid arguments. Args: <POINT> <MAJOR_MODE> <FILE_PATH>")
    }

    fn read_contents(point: usize, major_mode: &str, file_path: &str) -> Result<OffsetContents> {
        if major_mode != "org-mode" {
            if file_path == "-" {
                return Ok(OffsetContents {
                    offset: 0,
                    contents: io::read_to_string(io::stdin())?,
                });
            } else {
                return Ok(OffsetContents {
                    offset: 0,
                    contents: fs::read_to_string(file_path)?,
                });
            }
        }

        let reader: Box<dyn Read> = if file_path == "-" {
            Box::new(io::stdin())
        } else {
            Box::new(File::open(file_path)?)
        };
        let reader = BufReader::new(reader);

        Self::org_src_block_contents(point, reader)
    }

    pub(crate) fn org_src_block_contents_from_string(
        point: usize,
        offset: usize,
        s: String,
    ) -> Result<OffsetContents> {
        let mut i = 0;
        let mut found_point = false;
        let mut open = None::<usize>;
        let mut close = None::<usize>;

        for line in IterateStringLines::new(&s) {
            let start = i;
            let end = i + line.len();
            let line = line.to_lowercase();
            let line = line.trim_start();

            if start <= point && end > point {
                found_point = true;
            }

            if line.len() >= BEGIN_SRC_BLOCK.len()
                && line[0..BEGIN_SRC_BLOCK.len()].starts_with(BEGIN_SRC_BLOCK)
            {
                if found_point {
                    bail!("{BEGIN_SRC_BLOCK} after point");
                }

                open = Some(end);
                close = None;
            } else if line.len() >= END_SRC_BLOCK.len()
                && line[0..END_SRC_BLOCK.len()].starts_with(END_SRC_BLOCK)
            {
                close = Some(start);

                if found_point {
                    break;
                }
            }

            i = end;
        }

        let Some(open) = open else {
            bail!("no open");
        };
        let Some(close) = close else {
            bail!("no close");
        };
        if close <= open {
            bail!("point is not within a src block");
        }

        Ok(OffsetContents {
            offset: open + offset,
            contents: (&s[open..close]).to_owned(),
        })
    }

    pub(crate) fn org_src_block_contents<T>(point: usize, mut reader: T) -> Result<OffsetContents>
    where
        T: BufRead,
    {
        let mut buffer = String::new();
        let mut i = 0;
        let mut found_point = false;
        let mut open = None::<usize>;
        let mut close = None::<usize>;
        let mut contents = String::new();

        while reader.read_line(&mut buffer)? > 0 {
            let start = i;
            let end = i + buffer.len();
            let line = buffer.to_lowercase();
            let line = line.trim_start();

            if start <= point && end > point {
                found_point = true;
            }

            if line.len() >= BEGIN_SRC_BLOCK.len()
                && line[0..BEGIN_SRC_BLOCK.len()].starts_with(BEGIN_SRC_BLOCK)
            {
                if found_point {
                    bail!("{BEGIN_SRC_BLOCK} after point");
                }

                open = Some(end);
                close = None;
                contents = String::new();
            } else if line.len() >= END_SRC_BLOCK.len()
                && line[0..END_SRC_BLOCK.len()].starts_with(END_SRC_BLOCK)
            {
                close = Some(start);

                if found_point {
                    break;
                }
            } else if open.is_some() {
                contents.push_str(&buffer);
            }

            if cfg!(debug_assertions) {
                let mut text = buffer.clone();

                let tag = if let Some(close) = close {
                    if close == start {
                        (BG_BLUE, "CLOSE )")
                    } else {
                        (BG_RED, "OUTSIDE")
                    }
                } else if let Some(open) = open {
                    if start <= point && end >= point {
                        let (before, after) = text.split_at(point - start);
                        let (point_char, after) = after.split_at(1);
                        text = format!("{before}{BG_YELLOW}{point_char}{RESET}{after}");

                        (BG_YELLOW, "POINT +")
                    } else if open == end {
                        (BG_BLUE, "OPEN  (")
                    } else {
                        (BG_GREEN, "INSIDE ")
                    }
                } else {
                    (BG_RED, "OUTSIDE")
                };

                eprint!("{BLACK}{}{}{RESET}  {text}", tag.0, tag.1);
            }

            i = end;
            buffer.clear();
        }

        let Some(open) = open else {
            bail!("no open");
        };
        let Some(close) = close else {
            bail!("no close");
        };
        if close <= open {
            bail!("point is not within a src block");
        }

        Ok(OffsetContents {
            offset: open,
            contents,
        })
    }
}
