use crate::parser::eval::Evaluator;

use super::*;

#[test]
fn test_parser() {
    let input = r#"
        S23: -> switch(M) | <- back(N, right)
        S25: -> back(L, left) | <- back(M, right)
        S26: -> none | <- back(M, left)
        S22: -> switch(N) | <- switch(O)
        S8: -> back(O, right) | <- none
        S21: -> back(O, left) | <- switch(P)
        S24: -> back(N, left) | <- back(P, right)
        S20: -> back(L, right) | <- back(P, left)
        S13: -> none | <- switch(L)
    "#;

    let result = parser().parse(input).into_result();

    match result {
        Ok(sections) => {
            for section in sections {
                println!("Section: {:?}", section);
            }
        }
        Err(errors) => {
            for error in errors {
                println!("Error: {}", error);
            }
        }
    }
}

#[test]
fn test_eval() {
    let input = r#"
        S23: -> switch(M) | <- back(N, right)
        S25: -> back(L, left) | <- back(M, right)
        S26: -> none | <- back(M, left)
        S22: -> switch(N) | <- switch(O)
        S8: -> back(O, right) | <- none
        S21: -> back(O, left) | <- switch(P)
        S24: -> back(N, left) | <- back(P, right)
        S20: -> back(L, right) | <- back(P, left)
        S13: -> none | <- switch(L)
    "#;

    let result = parser().parse(input).into_result();
    let section_defs = result.unwrap();

    let eval = Evaluator::new();
    let track = eval.evaluate(section_defs);

    match track {
        Ok(track) => {
            println!("Track: {:#?}", track);
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}
