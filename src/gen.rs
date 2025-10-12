use crate::bitboard::{Bitboard, LINE_AT_Y};
use crate::board::Board;
use crate::moves::{Move, SimpleMove};
use crate::patterns::*;
use crate::piece::Piece;

#[derive(Copy, Clone)]
struct OccupancyIterator(Bitboard);

impl Iterator for OccupancyIterator {
    type Item = Bitboard;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0 .0 & ((self.0 .0 as i64).wrapping_neg()) as u64;
        self.0 .0 &= !next;
        if next != 0 {
            Some(Bitboard(next))
        } else {
            None
        }
    }
}

impl Bitboard {
    fn occupied(&self) -> impl Iterator<Item = Bitboard> {
        OccupancyIterator(*self)
    }
}

impl Board {
    fn hyperbola_quintessence(occupancy: Bitboard, attacker: Bitboard, mask: Bitboard) -> Bitboard {
        let relevant = mask & occupancy;
        let fwd = relevant - Bitboard(attacker.0 << 1);
        let rev = (relevant.flip() - attacker.flip() - attacker.flip()).flip();
        mask & (fwd ^ rev)
    }

    pub fn rook_reach(&self, rook_position: Bitboard) -> Bitboard {
        // At this point I realize my grasp on the correspondence between bits and squares is rather
        // tenuous.
        let occupancy = self.occupancy();

        let (rook_file, rook_rank) = rook_position.coords();
        let rank_shift = 8 * (7 - rook_rank);
        let rank_occupancy: u64 = (occupancy.0 & (0b11111111 << rank_shift)) >> rank_shift;
        let rank_occupancy_reduced = ((rank_occupancy & 0b01111110) >> 1) as usize;
        let rank_attacks =
            Bitboard(RANK_ATTACKS[rook_file as usize][rank_occupancy_reduced].0 << rank_shift);

        let file_mask = FILES[rook_position];
        let file_attacks = Board::hyperbola_quintessence(occupancy, rook_position, file_mask);

        rank_attacks | file_attacks
    }

    pub fn bishop_reach(&self, bishop_position: Bitboard) -> Bitboard {
        let occupancy = self.occupancy();

        let mask_nw = NW_DIAGONALS[bishop_position];
        let nw_attacks = Board::hyperbola_quintessence(occupancy, bishop_position, mask_nw);
        let mask_sw = SW_DIAGONALS[bishop_position];
        let sw_attacks = Board::hyperbola_quintessence(occupancy, bishop_position, mask_sw);

        nw_attacks | sw_attacks
    }

    pub fn knight_reach(&self, knight_position: Bitboard) -> Bitboard {
        KNIGHT_ATTACKS[knight_position]
    }

    pub fn king_reach(&self, king_position: Bitboard) -> Bitboard {
        KING_ATTACKS[king_position]
    }

    pub fn queen_reach(&self, queen_position: Bitboard) -> Bitboard {
        self.bishop_reach(queen_position) | self.rook_reach(queen_position)
    }

    fn for_each_pawn_advance(&self, pos: Bitboard, process_move: &mut impl FnMut(SimpleMove)) {
        let mv_tgt = PAWN_SINGLE_MOVES[self.player as usize][pos];
        if (mv_tgt & !self.occupancy()).is_populated() {
            // Check whether we've reached the last rank
            let pieces: &[Piece] = if (mv_tgt & (LINE_AT_Y[0] | LINE_AT_Y[7])).is_populated() {
                &[Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen]
            } else {
                &[Piece::Pawn]
            };
            for piece in pieces {
                process_move(SimpleMove {
                    delete: pos | mv_tgt,
                    piece: *piece,
                    add: mv_tgt,
                })
            }
            // We do the double moves here since we already know the intermediate
            // square was unoccupied
            let mv2_tgt = PAWN_DOUBLE_MOVES[self.player as usize][pos];
            if (mv2_tgt & !self.occupancy()).is_populated() {
                process_move(SimpleMove {
                    delete: pos | mv2_tgt,
                    piece: Piece::Pawn,
                    add: mv2_tgt,
                })
            }
        }
    }

    fn for_each_pawn_capture(&self, pos: Bitboard, process_move: &mut impl FnMut(SimpleMove)) {
        let cap_tgt = PAWN_ATTACKS[self.player as usize][pos];
        // At most 2 captures; one on each side
        for tgt in (cap_tgt & self[self.player.opponent()]).occupied().take(2) {
            process_move(SimpleMove {
                delete: pos | tgt,
                piece: Piece::Pawn,
                add: tgt,
            })
        }
        let ep_info = PAWN_EP_INFO[self.player as usize][self.en_passant as usize];
        // I can't see a way to do e.p. without an extra conditional
        if (ep_info.source_squares & pos).is_populated() {
            process_move(SimpleMove {
                delete: pos | ep_info.kill_square,
                piece: Piece::Pawn,
                add: ep_info.target_square,
            })
        }
    }

    pub fn for_each_pawn_move(&self, process_move: &mut impl FnMut(SimpleMove)) {
        for pawn_pos in (self[self.player] & self[Piece::Pawn]).occupied() {
            self.for_each_pawn_advance(pawn_pos, process_move);
            self.for_each_pawn_capture(pawn_pos, process_move);
        }
    }

    pub fn for_each_knight_move(&self, process_move: &mut impl FnMut(SimpleMove)) {
        for knight_pos in (self[self.player] & self[Piece::Knight]).occupied() {
            for tgt in (self.knight_reach(knight_pos) & !self[self.player]).occupied() {
                process_move(SimpleMove {
                    delete: knight_pos | tgt,
                    piece: Piece::Knight,
                    add: tgt,
                })
            }
        }
    }

    pub fn for_each_bishop_move(&self, process_move: &mut impl FnMut(SimpleMove)) {
        for bishop_pos in (self[self.player] & self[Piece::Bishop]).occupied() {
            for tgt in (self.bishop_reach(bishop_pos) & !self[self.player]).occupied() {
                process_move(SimpleMove {
                    delete: bishop_pos | tgt,
                    piece: Piece::Bishop,
                    add: tgt,
                })
            }
        }
    }

    pub fn for_each_rook_move(&self, process_move: &mut impl FnMut(SimpleMove)) {
        for rook_pos in (self[self.player] & self[Piece::Rook]).occupied() {
            for tgt in (self.rook_reach(rook_pos) & !self[self.player]).occupied() {
                process_move(SimpleMove {
                    delete: rook_pos | tgt,
                    piece: Piece::Rook,
                    add: tgt,
                })
            }
        }
    }
    pub fn for_each_queen_move(&self, process_move: &mut impl FnMut(SimpleMove)) {
        for queen_pos in (self[self.player] & self[Piece::Queen]).occupied() {
            for tgt in (self.queen_reach(queen_pos) & !self[self.player]).occupied() {
                process_move(SimpleMove {
                    delete: queen_pos | tgt,
                    piece: Piece::Queen,
                    add: tgt,
                })
            }
        }
    }
    pub fn for_each_king_move(&self, process_move: &mut impl FnMut(SimpleMove)) {
        for king_pos in (self[self.player] & self[Piece::King]).occupied() {
            for tgt in (self.king_reach(king_pos) & !self[self.player]).occupied() {
                process_move(SimpleMove {
                    delete: king_pos | tgt,
                    piece: Piece::King,
                    add: tgt,
                })
            }
        }
    }
    pub fn for_each_pre_legal_castling_move(&self, process_move: &mut impl FnMut(Move)) {
        if self.short_castling_allowed(self.player) {
            if let Ok(mv) = self.castle_short() {
                process_move(mv)
            }
        }
        if self.long_castling_allowed(self.player) {
            if let Ok(mv) = self.castle_long() {
                process_move(mv)
            }
        }
    }

    pub fn for_each_piece_simple_move(&self, piece: Piece, process_move: &mut impl FnMut(SimpleMove)) {
        match piece {
            Piece::Pawn => self.for_each_pawn_move(process_move),
            Piece::Knight => self.for_each_knight_move(process_move),
            Piece::Bishop => self.for_each_bishop_move(process_move),
            Piece::Rook => self.for_each_rook_move(process_move),
            Piece::Queen => self.for_each_queen_move(process_move),
            Piece::King => self.for_each_king_move(process_move),
        }
    }
    // I can't come up with a way of abstracting this that's not potentially hard to
    // optimize. Packing it into an iterator will add a bunch of indirection since
    // we have to figure out what piece we're moving at runtime
    pub fn for_each_pre_legal_simple_move(&self, process_move: &mut impl FnMut(SimpleMove)) {
        self.for_each_pawn_move(process_move);
        self.for_each_knight_move(process_move);
        self.for_each_bishop_move(process_move);
        self.for_each_rook_move(process_move);
        self.for_each_queen_move(process_move);
        self.for_each_king_move(process_move);
    }
}
