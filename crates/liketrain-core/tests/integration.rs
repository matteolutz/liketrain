use chumsky::Parser;
use liketrain_core::{
    Direction, Route,
    parser::{eval::Evaluator, parser},
};

const TTL: &str = r#"

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

#[test]
fn test_route_validation() {
    let result = parser().parse(TTL).into_result();
    let track_defs = result.unwrap();

    let eval = Evaluator::default();
    let track = eval.evaluate(track_defs).unwrap();

    let r1 = Route::new([24_usize, 22, 21, 24], Direction::Forward);
    let r1_is_valid = r1.validate(&track);

    assert!(r1_is_valid);
    println!("Route 1 valid: {}", r1_is_valid);

    let r2 = Route::new(
        [24_usize, 22, 8, 3, 9, 10, 11, 13, 25, 23],
        Direction::Forward,
    );
    let r2_is_valid = r2.validate(&track);

    assert!(r2_is_valid);
    println!("Route 2 valid: {}", r2_is_valid);
}
