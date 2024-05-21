use regex::Regex;

#[derive(Debug)]
struct Ref {
    name: String,
}
#[derive(Debug)]
struct Parent {
    rev: Box<RefType>,
}
#[derive(Debug)]
struct Ancestor {
    rev: Box<RefType>,
    num: i32,
}

#[derive(Debug)]
enum RefType {
    Parent(Parent),
    Ancestor(Ancestor),
    Ref(Ref),
}

fn main() {
    let ref_type = parse("@~2^");
    // println!("{:?}", ref_type);
    dbg!(ref_type);
}

fn parse(hay: &str) -> RefType {
    let parent_re = Regex::new(r"^(.+)\^$").unwrap();
    let revision_re = Regex::new(r"^(.+)~(\d+)$").unwrap();

    if parent_re.is_match(hay) {
        let caps = parent_re.captures(hay).unwrap();
        let rev = caps.get(1).unwrap().as_str();

        let ref_type = parse(rev);
        let box_ref_type = Box::new(ref_type);
        return RefType::Parent(Parent { rev: box_ref_type });
    } else if revision_re.is_match(hay) {
        let caps = revision_re.captures(hay).unwrap();
        let rev = caps.get(1).unwrap().as_str();
        let num = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();

        let ref_type = parse(rev);
        let box_ref_type = Box::new(ref_type);
        return RefType::Ancestor(Ancestor {
            rev: box_ref_type,
            num,
        });
    } else {
        if hay == "@" {
            return RefType::Ref(Ref {
                name: "HEAD".to_string(),
            });
        }
        return RefType::Ref(Ref {
            name: hay.to_string(),
        });
    }
}
