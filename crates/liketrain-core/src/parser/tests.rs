use crate::{Direction, SectionTransition, parser::eval::Evaluator};

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

        S11:    -> back(K, left)        | <- switch(J)
        S10:    -> back(B, left)        | <- switch(K)
        S9:     -> switch(C)            | <- switch(B)
        S16:    -> switch(H)            | <- back(C, right)
        S14:    -> switch(I)            | <- back(H, right)
        S15:    -> none                 | <- back(H, left)
        S12:    -> back(K, right)       | <- back(I, left)

        switch(I, left) -> switch(J, right)
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

        S23:    -> switch(M)            | <- back(N, right)
        S25:    -> back(L, left)        | <- back(M, right)
        S26:    -> none                 | <- back(M, left)
        S22:    -> switch(N)            | <- switch(O)
        S8:     -> back(O, right)       | <- back(E, right)
        S21:    -> back(O, left)        | <- switch(P)
        S24:    -> back(N, left)        | <- back(P, right)
        S20:    -> back(L, right)       | <- back(P, left)
        S13:    -> back(J, left)        | <- switch(L)


        S11:    -> back(K, left)        | <- switch(J)
        S10:    -> back(B, left)        | <- switch(K)
        S9:     -> switch(C)            | <- switch(B)
        S16:    -> switch(H)            | <- back(C, right)
        S14:    -> switch(I)            | <- back(H, right)
        S15:    -> none                 | <- back(H, left)
        S12:    -> back(K, right)       | <- back(I, left)

        switch(I, right)                 -> switch(J, right)

        S3:     -> switch(E)            | <- switch(D)
        S4:     -> back(D, right)       | <- back(A, left)

        switch(D, left)                 -> switch(C, left)
        switch(A, right)                -> switch(B, right)

        S5:     -> switch(A)            | <- switch(G)
        S6:     -> back(G, right)       | <- none
        S7:     -> back(G, left)        | <- back(F, right)
        S1:     -> none                 | <- back(F, left)
        S2:     -> switch(F)            | <- back(E, left)


    "#;

    let result = parser().parse(input).into_result();
    let track_defs = result.unwrap();

    let eval = Evaluator::default();
    let track = eval.evaluate(track_defs);

    match track {
        Ok(track) => {
            println!("Track: {:#?}", track);

            let section_id = track.section_id("S14").unwrap();
            let section = track.section(&section_id).unwrap();

            let direction = Direction::Forward;

            let transitions = track.transitions(section_id, direction).unwrap();

            println!(
                "transitions from section {} going {}:\n{}",
                section.name(),
                direction,
                SectionTransition::pretty_print_iter(&transitions, &track)
            );
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}
