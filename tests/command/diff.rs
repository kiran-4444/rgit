use r_git::diff::{EditType, Myres};

#[test]
fn test_diff() {
    let a = r#"mod author;
            mod blob;
            mod commit;
            mod database;
            mod storable;
            mod tree;"#;

    let b = r#"mod author;
            mod blob;
            pub use self::commit::Commit;
            pub use self::database::{Content, Database, FileMode};
            mod tree;"#;

    let myres = Myres::new(a.to_string(), b.to_string());
    let hunks = myres.diff();

    assert_eq!(hunks.len(), 1);

    let hunk = &hunks[0];
    let (a_offset, b_offset) = hunk.header();

    assert_eq!(a_offset, vec![1, 6]);
    assert_eq!(b_offset, vec![1, 5]);

    assert_eq!(hunk.edits.len(), 8);

    let edit_types = hunk
        .edits
        .iter()
        .map(|edit| edit.edit_type.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        edit_types,
        vec![
            EditType::Equal,
            EditType::Equal,
            EditType::Remove,
            EditType::Remove,
            EditType::Remove,
            EditType::Add,
            EditType::Add,
            EditType::Equal
        ]
    );
}
