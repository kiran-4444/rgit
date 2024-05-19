use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub line: String,
    pub line_number: i32,
}

impl Line {
    fn new(line: String, line_number: i32) -> Line {
        Line { line, line_number }
    }
}

pub struct Myres {
    pub a: String,
    pub b: String,
}

impl Myres {
    pub fn new(a: String, b: String) -> Self {
        Self { a, b }
    }

    pub fn diff(&self) -> Vec<Hunk> {
        let a_lines: Vec<&str> = self.a.lines().collect();
        let b_lines: Vec<&str> = self.b.lines().collect();

        let a_lines = a_lines
            .iter()
            .enumerate()
            .map(|(i, line)| Line::new((*line).to_owned(), i as i32 + 1))
            .collect::<Vec<_>>();

        let b_lines = b_lines
            .iter()
            .enumerate()
            .map(|(i, line)| Line::new((*line).to_owned(), i as i32 + 1))
            .collect::<Vec<_>>();

        let trace = diff(&a_lines, &b_lines);
        let ans = backtrack(trace.0, &a_lines, &b_lines);

        let edits = render(&a_lines, &b_lines, ans);

        let edits = edits.clone();

        let hunks = Hunk::filter(&edits);
        hunks
    }
}

#[derive(Debug, Clone)]
pub struct Hunk {
    a_start: i32,
    b_start: i32,
    pub edits: Vec<Edit>,
}

const HUNK_CONTEXT: usize = 3;

impl Hunk {
    fn new(a_start: i32, b_start: i32, edits: Vec<Edit>) -> Hunk {
        Hunk {
            a_start,
            b_start,
            edits,
        }
    }

    pub fn header(&self) -> (Vec<usize>, Vec<usize>) {
        let a_offset = self.offsets_for(|edit| edit.a_line.clone(), self.a_start as usize);
        let b_offset = self.offsets_for(|edit| edit.b_line.clone(), self.b_start as usize);

        (a_offset, b_offset)
    }

    fn offsets_for<F>(&self, line_type: F, default: usize) -> Vec<usize>
    where
        F: Fn(&Edit) -> Option<Line>,
    {
        let lines = self
            .edits
            .iter()
            .filter_map(|edit| line_type(edit))
            .collect::<Vec<_>>();

        let start = lines
            .first()
            .map_or(default, |line| line.line_number as usize);
        vec![start, lines.len()]
    }

    fn build(hunk: &mut Hunk, edits: &Vec<Edit>, offset: i32) -> i32 {
        let mut counter: i32 = -1;
        let mut offset = offset;

        while counter != 0 {
            if offset >= 0 && counter > 0 {
                let edit = edits[offset as usize].clone();
                hunk.edits.push(edit);
            }

            offset += 1;

            if offset >= edits.len() as i32 {
                break;
            }

            if offset as i32 + HUNK_CONTEXT as i32 >= edits.len() as i32 {
                continue;
            }

            match edits[(offset as i32 + HUNK_CONTEXT as i32) as usize].edit_type {
                EditType::Equal => {
                    counter -= 1;
                }
                EditType::Add | EditType::Remove => {
                    counter = 2 * (HUNK_CONTEXT as i32) + 1;
                }
            }
        }

        offset
    }

    fn filter(edits: &Vec<Edit>) -> Vec<Hunk> {
        let mut hunks = vec![];
        let mut offset: i32 = 0;

        loop {
            while (offset as i32) < edits.len() as i32
                && edits[offset as usize].edit_type == EditType::Equal
            {
                offset += 1;
            }

            if (offset as i32) >= edits.len() as i32 {
                return hunks;
            }

            offset -= (HUNK_CONTEXT as i32) + 1;

            let a_start = if offset < 0 {
                0
            } else {
                edits[offset as usize]
                    .a_line
                    .clone()
                    .expect("found none a_line")
                    .line_number
            };
            let b_start = if offset < 0 {
                0
            } else {
                edits[offset as usize]
                    .b_line
                    .clone()
                    .expect("found none b_line")
                    .line_number
            };

            let mut hunk = Hunk::new(a_start, b_start, vec![]);
            offset = Hunk::build(&mut hunk, edits, offset);
            hunks.push(hunk);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EditType {
    Add,
    Remove,
    Equal,
}

#[derive(Debug, Clone)]
pub struct Edit {
    pub edit_type: EditType,
    pub a_line: Option<Line>,
    pub b_line: Option<Line>,
}

fn render<'a>(
    a_lines: &'a Vec<Line>,
    b_lines: &'a Vec<Line>,
    ans: Vec<(i32, i32, i32, i32)>,
) -> Vec<Edit> {
    let mut diff = vec![];

    for (prev_x, prev_y, x, y) in ans {
        let mut a_line = " ";
        let mut b_line = " ";
        let mut a_line_number = -1;
        let mut b_line_number = -1;
        if prev_x < a_lines.len() as i32 {
            a_line = a_lines[prev_x as usize].line.as_str();
            a_line_number = a_lines[prev_x as usize].line_number;
        }
        if prev_y < b_lines.len() as i32 {
            b_line = b_lines[prev_y as usize].line.as_str();
            b_line_number = b_lines[prev_y as usize].line_number;
        }

        if x == prev_x {
            diff.push(Edit {
                edit_type: EditType::Add,
                a_line: None,
                b_line: Some(Line::new(b_line.to_owned(), b_line_number)),
            });
        } else if y == prev_y {
            diff.push(Edit {
                edit_type: EditType::Remove,
                a_line: Some(Line::new(a_line.to_owned(), a_line_number)),
                b_line: None,
            });
        } else {
            diff.push(Edit {
                edit_type: EditType::Equal,
                a_line: Some(Line::new(a_line.to_owned(), a_line_number)),
                b_line: Some(Line::new(b_line.to_owned(), b_line_number)),
            });
        }
    }

    diff.reverse();
    diff
}

fn backtrack(
    trace: Vec<Vec<i32>>,
    a_lines: &Vec<Line>,
    b_lines: &Vec<Line>,
) -> Vec<(i32, i32, i32, i32)> {
    let mut x = a_lines.len() as i32;
    let mut y = b_lines.len() as i32;

    let mut path = vec![];
    for (d, v) in trace.iter().enumerate().rev() {
        let v_myvec = MyVec(v.clone());
        let d = d as i32;
        let k = x - y;
        let prev_k = if k == -d || (k != d && v_myvec[k - 1] < v_myvec[k + 1]) {
            k + 1
        } else {
            k - 1
        };

        let prev_x = v_myvec[prev_k];
        let prev_y = prev_x - prev_k;

        while x > prev_x && y > prev_y {
            path.push((x - 1, y - 1, x, y));
            x = x - 1;
            y = y - 1;
        }

        if d > 0 {
            path.push((prev_x, prev_y, x, y));
        }

        x = prev_x;
        y = prev_y;
    }

    path
}

#[derive(Debug)]
struct MyVec<T>(Vec<T>);
impl<T> Index<i32> for MyVec<T> {
    type Output = T;

    fn index(&self, index: i32) -> &Self::Output {
        let idx = if index < 0 {
            self.0.len() as i32 + index
        } else {
            index
        };

        &self.0[idx as usize]
    }
}

impl<T> IndexMut<i32> for MyVec<T> {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        let idx = if index < 0 {
            self.0.len() as i32 + index
        } else {
            index
        };

        &mut self.0[idx as usize]
    }
}

// implement index extension such that it uses negative indexing

fn diff(a: &Vec<Line>, b: &Vec<Line>) -> (Vec<Vec<i32>>, i32) {
    let v = vec![-1; 2 * (a.len() + b.len()) + 1];
    let mut v = MyVec(v);
    v[1] = 0;
    let mut trace = vec![];

    for d in 0..=(a.len() + b.len()) as i32 {
        trace.push(v.0.clone());
        for k in (-d..=d).step_by(2) {
            let mut x = if k == -d || (k != d && v[k - 1] < v[k + 1]) {
                v[k + 1]
            } else {
                v[k - 1] + 1
            };

            let mut y = x - k;

            while x < a.len() as i32
                && y < b.len() as i32
                && a[x as usize].line == b[y as usize].line
            {
                x += 1;
                y += 1;
            }

            v[k] = x;
            if x >= a.len() as i32 && y >= b.len() as i32 {
                return (trace, d);
            }
        }
    }

    return (trace, -1);
}
