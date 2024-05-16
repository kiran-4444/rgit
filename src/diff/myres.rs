use colored::{ColoredString, Colorize};
use std::ops::{Index, IndexMut};

pub struct Myres {
    pub a: String,
    pub b: String,
}

impl Myres {
    pub fn new(a: String, b: String) -> Self {
        Self { a, b }
    }

    pub fn diff(&self) -> Vec<ColoredString> {
        let a_lines: Vec<&str> = self.a.lines().collect();
        let b_lines: Vec<&str> = self.b.lines().collect();
        let trace = diff(&a_lines, &b_lines);
        let ans = backtrack(trace.0, &a_lines, &b_lines);
        render(&a_lines, &b_lines, ans)
    }
}

fn render(
    a_lines: &Vec<&str>,
    b_lines: &Vec<&str>,
    ans: Vec<(i32, i32, i32, i32)>,
) -> Vec<ColoredString> {
    let mut diff = vec![];

    for (prev_x, prev_y, x, y) in ans {
        let mut a_line = " ";
        let mut b_line = " ";
        if prev_x < a_lines.len() as i32 {
            a_line = a_lines[prev_x as usize];
        }
        if prev_y < b_lines.len() as i32 {
            b_line = b_lines[prev_y as usize];
        }

        if x == prev_x {
            diff.push(format!("+ {}", b_line).green());
        } else if y == prev_y {
            diff.push(format!("- {}", a_line).red());
        } else {
            diff.push(format!("  {}", a_line).normal());
        }
    }

    diff.reverse();
    diff
}

fn backtrack(
    trace: Vec<Vec<i32>>,
    a_lines: &Vec<&str>,
    b_lines: &Vec<&str>,
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

fn diff(a: &Vec<&str>, b: &Vec<&str>) -> (Vec<Vec<i32>>, i32) {
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

            while x < a.len() as i32 && y < b.len() as i32 && a[x as usize] == b[y as usize] {
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
