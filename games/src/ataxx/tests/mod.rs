use crate::ataxx::Position;
use crate::perft;
use std::str::FromStr;

macro_rules! perft_test {
    ($name:ident $pos:literal $depth:literal $nodes:literal) => {
        #[test]
        fn $name() {
            let position = Position::from_str($pos).unwrap();
            assert_eq!(perft::<true, true, _>(position, $depth), $nodes)
        }
    };
}

// Tests taken from libataxx

perft_test!(position_01_x "7/7/7/7/7/7/7 x 0 1" 4 0);
perft_test!(position_01_o "7/7/7/7/7/7/7 o 0 1" 4 0);
perft_test!(position_02_x "x5o/7/7/7/7/7/o5x x 0 1" 5 4752668);
perft_test!(position_02_o "x5o/7/7/7/7/7/o5x o 0 1" 5 4752668);
perft_test!(position_03_x "x5o/7/2-1-2/7/2-1-2/7/o5x x 0 1" 5 2266352);
perft_test!(position_03_o "x5o/7/2-1-2/7/2-1-2/7/o5x o 0 1" 5 2266352);
perft_test!(position_04_x "x5o/7/2-1-2/3-3/2-1-2/7/o5x x 0 1" 5 2114588);
perft_test!(position_04_o "x5o/7/2-1-2/3-3/2-1-2/7/o5x o 0 1" 5 2114588);
perft_test!(position_05_x "x5o/7/3-3/2-1-2/3-3/7/o5x x 0 1" 5 3639856);
perft_test!(position_05_o "x5o/7/3-3/2-1-2/3-3/7/o5x o 0 1" 5 3639856);
perft_test!(position_06_x "7/7/7/7/ooooooo/ooooooo/xxxxxxx x 0 1" 5 452980);
perft_test!(position_06_o "7/7/7/7/ooooooo/ooooooo/xxxxxxx o 0 1" 4 452980);
perft_test!(position_07_x "7/7/7/7/xxxxxxx/xxxxxxx/ooooooo x 0 1" 4 452980);
perft_test!(position_07_o "7/7/7/7/xxxxxxx/xxxxxxx/ooooooo o 0 1" 5 452980);
perft_test!(position_08_x "7/7/7/2x1o2/7/7/7 x 0 1" 5 4266992);
perft_test!(position_08_o "7/7/7/2x1o2/7/7/7 o 0 1" 5 4266992);
perft_test!(position_09_x "x5o/7/7/7/7/7/o5x x 100 1" 5 0);
perft_test!(position_09_o "x5o/7/7/7/7/7/o5x o 100 1" 5 0);

// TODO: Deal with disjointing blockers
perft_test!(position_10_x "7/7/7/7/-------/-------/x5o x 0 1" 6 175); // 174 ^^
perft_test!(position_10_o "7/7/7/7/-------/-------/x5o o 0 1" 6 175); // 174 ^^
