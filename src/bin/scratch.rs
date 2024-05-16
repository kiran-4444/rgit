use std::ops::{Index, IndexMut};

fn main() {
    let a = std::env::args().nth(1).unwrap();
    let b = std::env::args().nth(2).unwrap();
    let ans = diff(&a, &b);
    // println!("{:?}, {:?}", ans.0, ans.1);
    let ans = backtrack(ans.0, &a, &b);
    // println!("{:?}", ans);
    let ans = render(&a, &b, ans);
    println!("{}", ans);
}

fn render(a: &str, b: &str, ans: Vec<(i32, i32, i32, i32)>) -> String {
    let mut diff = vec![];

    for (prev_x, prev_y, x, y) in ans {
        let mut a_line = ' ';
        let mut b_line = ' ';
        if prev_x < a.len() as i32 {
            a_line = a.chars().nth(prev_x as usize).unwrap();
        }
        if prev_y < b.len() as i32 {
            b_line = b.chars().nth(prev_y as usize).unwrap();
        }

        if x == prev_x {
            diff.push(format!("+ {}", b_line));
        } else if y == prev_y {
            diff.push(format!("- {}", a_line));
        } else {
            diff.push(format!("  {}", a_line));
        }
    }

    diff.reverse();
    diff.join("\n")
}

fn backtrack(trace: Vec<Vec<i32>>, a: &str, b: &str) -> Vec<(i32, i32, i32, i32)> {
    let mut x = a.len() as i32;
    let mut y = b.len() as i32;

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

fn diff(a: &str, b: &str) -> (Vec<Vec<i32>>, i32) {
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
                && a.chars().nth(x as usize) == b.chars().nth(y as usize)
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
