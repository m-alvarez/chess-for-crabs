use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::patterns::{
    FILES, KING_ATTACKS, KNIGHT_ATTACKS, NW_DIAGONALS, RANK_ATTACKS, SW_DIAGONALS,
};
use crate::piece::{Color, Piece};

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

    #[inline(never)]
    pub fn rook_reach(&self, rook_position: Bitboard) -> Bitboard {
        // At this point I realize my grasp on the correspondence between bits and squares is rather
        // tenuous.
        let occupancy = self.occupancy();

        let (rook_file, rook_rank) = rook_position.coords();
        let rank_shift = 8 * (7 - rook_rank);
        let rank_occupancy: u64 = (occupancy.0 & (0b11111111 << rank_shift)) >> rank_shift;
        let rank_occupancy_reduced = ((rank_occupancy & 0b01111110) >> 1) as usize;
        let rank_attacks =
            Bitboard(RANK_ATTACKS[rook_file][rank_occupancy_reduced].0 << rank_shift);

        let file_mask = FILES[rook_position];
        let file_attacks = Board::hyperbola_quintessence(occupancy, rook_position, file_mask);

        rank_attacks | file_attacks
    }

    #[inline(never)]
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

    fn piece_positions(&self, piece: Piece, color: Color) -> OccupancyIterator {
        OccupancyIterator(self[color] & self[piece])
    }
}
