use chumsky::Parser;
use liketrain_core::{
    Controller, ControllerConfig, Direction, Route, Train,
    comm::SerialControllerHardwareCommunication,
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

    let Some(r1) = Route::new([24_usize, 22, 21, 24], Direction::Forward, &track) else {
        assert!(false, "Route 1 failed");
        return;
    };

    println!("Route 1 valid: {}", r1.pretty_print(&track));

    let Some(r2) = Route::new(
        [24_usize, 22, 8, 3, 9, 10, 11, 13, 25, 23],
        Direction::Forward,
        &track,
    ) else {
        assert!(false, "Route 2 failed");
        return;
    };

    println!("Route 2 valid: {}", r2.pretty_print(&track));
}

#[test]
fn test_controller() {
    #[cfg(debug_assertions)]
    {
        if std::env::var("RUST_LOG").is_err() {
            unsafe { std::env::set_var("RUST_LOG", "debug") }
        }
    }

    env_logger::init();

    log::debug!("testing logger");

    let result = parser().parse(TTL).into_result();
    let track_defs = result.unwrap();

    let eval = Evaluator::default();
    let track = eval.evaluate(track_defs).unwrap();

    let Some(r1) = Route::new([24_usize, 22, 21, 24], Direction::Forward, &track) else {
        assert!(false, "Route 1 failed");
        return;
    };

    println!("Route 1 valid: {}", r1.pretty_print(&track));

    let hardware_comm = SerialControllerHardwareCommunication::new("COM3", 115200);

    let controller_config = ControllerConfig {
        track,
        trains: [(1_u32.into(), Train::from_route("RE5".into(), r1))].into(),
    };

    let controller = Controller::new(controller_config, hardware_comm);
    controller.start().unwrap();
}
