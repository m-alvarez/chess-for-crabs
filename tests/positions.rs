use std::fmt::{Debug, Formatter};
use std::fs::read_to_string;

use chess_for_crabs::*;
use game::Game;
use moves::AlgebraicMove;
use board::IllegalMove;

enum Error {
    IOError(std::io::Error),
    ParseError(String),
    IllegalMove(Game, AlgebraicMove, IllegalMove),
}

impl Debug for Error {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::IllegalMove(game, mv, illegal) => {
                writeln!(fmt, "Move {mv} is illegal in the position below:")?;
                writeln!(fmt, "{}", game.board.fen())?;
                writeln!(fmt, "{}", game.board)?;
                writeln!(fmt, "Reason: {}", illegal.as_str())
            }
            Error::ParseError(s) => {
                writeln!(fmt, "Error::ParseError({s:?})")
            }
            Error::IOError(err) => {
                writeln!(fmt, "Error::IOError({err:?})")
            }
        }
    }
}

fn test_position(game_no: usize) -> Result<(), Error> {
    let filename = format!("games/game_{game_no}.pgn");
    let moves = match read_to_string(filename) {
        Ok(s) => s,
        Err(err) => return Err(Error::IOError(err)),
    };
    let mut game = Game::new();

    for move_str in moves.split(' ') {
        let alg = AlgebraicMove::parse(&move_str).ok_or(Error::ParseError(move_str.to_string()))?;
        let mv = match game.board.is_legal(&alg) {
            Ok(mv) => mv,
            Err(illegal) => return Err(Error::IllegalMove(game, alg, illegal)),
        };
        game.make_move(&alg, &mv)
    }
    Ok(())
}

#[test] fn test_position_1() -> Result<(), Error> { test_position(1) }
#[test] fn test_position_2() -> Result<(), Error> { test_position(2) }
#[test] fn test_position_3() -> Result<(), Error> { test_position(3) }
#[test] fn test_position_4() -> Result<(), Error> { test_position(4) }
#[test] fn test_position_5() -> Result<(), Error> { test_position(5) }
#[test] fn test_position_6() -> Result<(), Error> { test_position(6) }
#[test] fn test_position_7() -> Result<(), Error> { test_position(7) }
#[test] fn test_position_8() -> Result<(), Error> { test_position(8) }
#[test] fn test_position_9() -> Result<(), Error> { test_position(9) }
#[test] fn test_position_10() -> Result<(), Error> { test_position(10) }
#[test] fn test_position_11() -> Result<(), Error> { test_position(11) }
#[test] fn test_position_12() -> Result<(), Error> { test_position(12) }
#[test] fn test_position_13() -> Result<(), Error> { test_position(13) }
#[test] fn test_position_14() -> Result<(), Error> { test_position(14) }
#[test] fn test_position_15() -> Result<(), Error> { test_position(15) }
#[test] fn test_position_16() -> Result<(), Error> { test_position(16) }
#[test] fn test_position_17() -> Result<(), Error> { test_position(17) }
#[test] fn test_position_18() -> Result<(), Error> { test_position(18) }
#[test] fn test_position_19() -> Result<(), Error> { test_position(19) }
#[test] fn test_position_20() -> Result<(), Error> { test_position(20) }
#[test] fn test_position_21() -> Result<(), Error> { test_position(21) }
#[test] fn test_position_22() -> Result<(), Error> { test_position(22) }
#[test] fn test_position_23() -> Result<(), Error> { test_position(23) }
#[test] fn test_position_24() -> Result<(), Error> { test_position(24) }
#[test] fn test_position_25() -> Result<(), Error> { test_position(25) }
#[test] fn test_position_26() -> Result<(), Error> { test_position(26) }
#[test] fn test_position_27() -> Result<(), Error> { test_position(27) }
#[test] fn test_position_28() -> Result<(), Error> { test_position(28) }
#[test] fn test_position_29() -> Result<(), Error> { test_position(29) }
#[test] fn test_position_30() -> Result<(), Error> { test_position(30) }
#[test] fn test_position_31() -> Result<(), Error> { test_position(31) }
#[test] fn test_position_32() -> Result<(), Error> { test_position(32) }
#[test] fn test_position_33() -> Result<(), Error> { test_position(33) }
#[test] fn test_position_34() -> Result<(), Error> { test_position(34) }
#[test] fn test_position_35() -> Result<(), Error> { test_position(35) }
#[test] fn test_position_36() -> Result<(), Error> { test_position(36) }
#[test] fn test_position_37() -> Result<(), Error> { test_position(37) }
#[test] fn test_position_38() -> Result<(), Error> { test_position(38) }
#[test] fn test_position_39() -> Result<(), Error> { test_position(39) }
#[test] fn test_position_40() -> Result<(), Error> { test_position(40) }
#[test] fn test_position_41() -> Result<(), Error> { test_position(41) }
#[test] fn test_position_42() -> Result<(), Error> { test_position(42) }
#[test] fn test_position_43() -> Result<(), Error> { test_position(43) }
#[test] fn test_position_44() -> Result<(), Error> { test_position(44) }
#[test] fn test_position_45() -> Result<(), Error> { test_position(45) }
#[test] fn test_position_46() -> Result<(), Error> { test_position(46) }
#[test] fn test_position_47() -> Result<(), Error> { test_position(47) }
#[test] fn test_position_48() -> Result<(), Error> { test_position(48) }
#[test] fn test_position_49() -> Result<(), Error> { test_position(49) }
#[test] fn test_position_50() -> Result<(), Error> { test_position(50) }
#[test] fn test_position_51() -> Result<(), Error> { test_position(51) }
#[test] fn test_position_52() -> Result<(), Error> { test_position(52) }
#[test] fn test_position_53() -> Result<(), Error> { test_position(53) }
#[test] fn test_position_54() -> Result<(), Error> { test_position(54) }
#[test] fn test_position_55() -> Result<(), Error> { test_position(55) }
#[test] fn test_position_56() -> Result<(), Error> { test_position(56) }
#[test] fn test_position_57() -> Result<(), Error> { test_position(57) }
#[test] fn test_position_58() -> Result<(), Error> { test_position(58) }
#[test] fn test_position_59() -> Result<(), Error> { test_position(59) }
#[test] fn test_position_60() -> Result<(), Error> { test_position(60) }
#[test] fn test_position_61() -> Result<(), Error> { test_position(61) }
#[test] fn test_position_62() -> Result<(), Error> { test_position(62) }
#[test] fn test_position_63() -> Result<(), Error> { test_position(63) }
#[test] fn test_position_64() -> Result<(), Error> { test_position(64) }
#[test] fn test_position_65() -> Result<(), Error> { test_position(65) }
#[test] fn test_position_66() -> Result<(), Error> { test_position(66) }
#[test] fn test_position_67() -> Result<(), Error> { test_position(67) }
#[test] fn test_position_68() -> Result<(), Error> { test_position(68) }
#[test] fn test_position_69() -> Result<(), Error> { test_position(69) }
#[test] fn test_position_70() -> Result<(), Error> { test_position(70) }
#[test] fn test_position_71() -> Result<(), Error> { test_position(71) }
#[test] fn test_position_72() -> Result<(), Error> { test_position(72) }
#[test] fn test_position_73() -> Result<(), Error> { test_position(73) }
#[test] fn test_position_74() -> Result<(), Error> { test_position(74) }
#[test] fn test_position_75() -> Result<(), Error> { test_position(75) }
#[test] fn test_position_76() -> Result<(), Error> { test_position(76) }
#[test] fn test_position_77() -> Result<(), Error> { test_position(77) }
#[test] fn test_position_78() -> Result<(), Error> { test_position(78) }
#[test] fn test_position_79() -> Result<(), Error> { test_position(79) }
#[test] fn test_position_80() -> Result<(), Error> { test_position(80) }
#[test] fn test_position_81() -> Result<(), Error> { test_position(81) }
#[test] fn test_position_82() -> Result<(), Error> { test_position(82) }
#[test] fn test_position_83() -> Result<(), Error> { test_position(83) }
#[test] fn test_position_84() -> Result<(), Error> { test_position(84) }
#[test] fn test_position_85() -> Result<(), Error> { test_position(85) }
#[test] fn test_position_86() -> Result<(), Error> { test_position(86) }
#[test] fn test_position_87() -> Result<(), Error> { test_position(87) }
#[test] fn test_position_88() -> Result<(), Error> { test_position(88) }
#[test] fn test_position_89() -> Result<(), Error> { test_position(89) }
#[test] fn test_position_90() -> Result<(), Error> { test_position(90) }
#[test] fn test_position_91() -> Result<(), Error> { test_position(91) }
#[test] fn test_position_92() -> Result<(), Error> { test_position(92) }
#[test] fn test_position_93() -> Result<(), Error> { test_position(93) }
#[test] fn test_position_94() -> Result<(), Error> { test_position(94) }
#[test] fn test_position_95() -> Result<(), Error> { test_position(95) }
#[test] fn test_position_96() -> Result<(), Error> { test_position(96) }
#[test] fn test_position_97() -> Result<(), Error> { test_position(97) }
#[test] fn test_position_98() -> Result<(), Error> { test_position(98) }
#[test] fn test_position_99() -> Result<(), Error> { test_position(99) }
#[test] fn test_position_100() -> Result<(), Error> { test_position(100) }
