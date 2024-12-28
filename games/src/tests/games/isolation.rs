use crate::common::perft::perft;
use crate::games::isolation::Position;
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

perft_test!(startpos_1 "--------/--------/p-------/-------P/--------/-------- w 1" 1 225);
perft_test!(startpos_2 "--------/--------/p-------/-------P/--------/-------- w 1" 2 47300);
perft_test!(startpos_3 "--------/--------/p-------/-------P/--------/-------- w 1" 3 10998540);
perft_test!(startpos_4 "--------/--------/p-------/-------P/--------/-------- w 1" 4 2434427562);
perft_test!(startpos_5 "--------/--------/p-------/-------P/--------/-------- w 1" 5 561436288000);
