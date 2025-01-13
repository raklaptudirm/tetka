use super::stats::{Model, Score};

#[test]
pub fn stats() {
    // ll, ld, dd/wl, wd, ww, elo0, elo1, elo, delta, llr
    // Values taken from random OpenBench instances.
    let tests: Vec<(usize, usize, usize, usize, usize, f64, f64, f64, f64, f64)> = vec![
        (47, 130, 233, 140, 46, -3.00, 1.00, 2.33, 14.49, 0.16),
        (119, 2924, 6214, 2887, 96, 0.00, 3.00, -1.18, 2.27, -2.29),
        (29, 476, 1098, 656, 39, 0.00, 3.00, 15.13, 5.53, 2.94),
        (136, 3385, 7009, 3497, 126, -3.00, 1.00, 1.13, 2.14, 2.94),
        (28, 628, 1092, 497, 26, 0.00, 3.00, -10.33, 5.48, -2.31),
        //(18, 192, 208, 64, 2, 0.00, 3.00, -57.96, 12.28, -2.30),
        //(239, 27, 2, 0, 0, 0.00, 3.00, -610.44, 77.83, -3.20),
        //(60, 78, 261, 117, 135, 0.00, 5.00, 50.79, 16.33, 3.01),
    ];

    for test in tests {
        let x = Score::new(test.0, test.1, 0, test.2, test.3, test.4);

        println!("Expected Results:");
        println!("ELO   | {:.2} +- {:.2}", test.7, test.8);
        println!("LLR   | {:.2} [{:.2}, {:.2}]", test.9, test.5, test.6);
        println!();

        let model = Model::Pentanomial;

        let llr = model.llr(x, test.5, test.6);
        let (elo, delta) = model.elo(x, 0.05);

        println!("Calculated Results:");
        println!("ELO   | {:.2} +- {:.2}", elo, delta);
        println!("LLR   | {:.2} [{:.2}, {:.2}]", llr, test.5, test.6);
        println!();

        macro_rules! assert_delta {
            ($x:expr, $y:expr, $d:expr) => {
                assert!(($x - $y).abs() < $d);
            };
        }

        assert_delta!(llr, test.9, 0.01);
        assert_delta!(elo, test.7, 0.01);
        assert_delta!(delta, test.8, 0.01);
    }
}
