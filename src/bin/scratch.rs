use std::ops::{Index, IndexMut};

fn main() {
    let a = std::env::args().nth(1).unwrap();
    let b = std::env::args().nth(2).unwrap();
    let ans = diff(&a, &b);
    println!("{}", ans);
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

fn diff(a: &str, b: &str) -> i32 {
    let v = vec![0; 2 * (a.len() + b.len()) + 1];
    let mut v = MyVec(v);

    for d in 0..=(a.len() + b.len() + 1) as i32 {
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
                return d;
            }
        }
    }

    return -1;
}
