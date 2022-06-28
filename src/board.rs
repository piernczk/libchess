// Copyright (C) 2022 Pierniczek
// 
// This file is part of libchess.
//
// libchess is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// libchess is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more
// details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with libchess. If not, see <https://www.gnu.org/licenses/>.

use bitvec::prelude::*;
use crate::piece::*;

pub type Bitboard = BitArr!(for 64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    #[inline]
    pub const fn new(file: u8, rank: u8) -> Self {
        assert!(file < 8);
        assert!(rank < 8);
        Self(file + rank * 8)
    }

    #[inline]
    pub const fn from_index(idx: u8) -> Self {
        assert!(idx < 64);
        Self(idx)
    }

    #[inline]
    pub fn index(self) -> u8 {
        self.0
    }

    #[inline]
    pub fn file(self) -> u8 {
        self.0 & 0b0000_0111
    }

    #[inline]
    pub fn rank(self) -> u8 {
        self.0 >> 3
    }

    #[inline]
    pub fn set_index(&mut self, idx: u8) {
        assert!(idx < 64);
        self.0 = idx;
    }

    #[inline]
    pub fn set_file(&mut self, file: u8) {
        assert!(file < 8);
        self.0 &= 0b0011_1000 | file;
    }

    #[inline]
    pub fn set_rank(&mut self, rank: u8) {
        assert!(rank < 8);
        self.0 &= 0b0000_0111 | (rank << 3);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SquareInfo {
    pub pl: Player,
    pub pc: Piece,
}

impl SquareInfo {
    #[inline]
    pub const fn new(pl: Player, pc: Piece) -> Self {
        Self { pl, pc }
    }
}

impl From<SquareInfo> for (Player, Piece) {
    fn from(sqi: SquareInfo) -> Self {
        (sqi.pl, sqi.pc)
    }
}

impl From<(Player, Piece)> for SquareInfo {
    fn from(t: (Player, Piece)) -> Self {
        Self {
            pl: t.0,
            pc: t.1,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    color: Bitboard,
    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    kings: Bitboard,
}

impl Board {
    pub const fn new() -> Self {
        Self {
            color: Bitboard::ZERO,
            pawns: Bitboard::ZERO,
            knights: Bitboard::ZERO,
            bishops: Bitboard::ZERO,
            rooks: Bitboard::ZERO,
            queens: Bitboard::ZERO,
            kings: Bitboard::ZERO,
        }
    }

    pub fn occupied(&self, sq: Square) -> bool {
        let idx = sq.index() as usize;

        self.pawns[idx]
            || self.knights[idx]
            || self.bishops[idx]
            || self.rooks[idx]
            || self.queens[idx]
            || self.kings[idx]
    }

    pub fn get_color(&self, sq: Square) -> Option<Player> {
        if self.occupied(sq) {
            Some(self.get_color_unchecked(sq))
        } else {
            None
        }
    }

    #[inline]
    pub fn get_color_unchecked(&self, sq: Square) -> Player {
        self.color[sq.index() as usize].into()
    }

    pub fn get_piece(&self, sq: Square) -> Option<Piece> {
        let idx = sq.index() as usize;

        Some(if self.pawns[idx] {
            Piece::Pawn
        } else if self.knights[idx] {
            Piece::Knight
        } else if self.bishops[idx] {
            Piece::Bishop
        } else if self.rooks[idx] {
            Piece::Rook
        } else if self.queens[idx] {
            Piece::Queen
        } else if self.pawns[idx] {
            Piece::King
        } else {
            return None
        })
    }

    pub fn square_info(&self, sq: Square) -> Option<SquareInfo> {
        match self.get_piece(sq) {
            Some(pc) => Some(SquareInfo::new(self.get_color_unchecked(sq), pc)),
            None => None
        }
    }

    #[inline]
    pub fn set_color(&mut self, sq: Square, pl: Player) {
        self.color.set(sq.index() as usize, pl.into());
    }

    pub fn set_piece(&mut self, sq: Square, pc: Piece) {
        let idx = sq.index() as usize;
        self.unset_square(sq);

        match pc {
            Piece::Pawn => self.pawns.set(idx, true),
            Piece::Knight => self.knights.set(idx, true),
            Piece::Bishop => self.bishops.set(idx, true),
            Piece::Rook => self.rooks.set(idx, true),
            Piece::Queen => self.queens.set(idx, true),
            Piece::King => self.kings.set(idx, true),
        }
    }

    pub fn unset_square(&mut self, sq: Square) {
        let idx = sq.index() as usize;

        self.pawns.set(idx, false);
        self.knights.set(idx, false);
        self.bishops.set(idx, false);
        self.rooks.set(idx, false);
        self.queens.set(idx, false);
        self.kings.set(idx, false);
    }

    #[inline]
    pub fn set_square_info(&mut self, sq: Square, sqi: SquareInfo) {
        self.set_color(sq, sqi.pl);
        self.set_piece(sq, sqi.pc);
    }

    #[inline]
    pub fn set_square(&mut self, sq: Square, pl: Player, pc: Piece) {
        self.set_color(sq, pl);
        self.set_piece(sq, pc);
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        let soccupied = self.pawns | self.knights | self.bishops | self.rooks | self.queens | self.kings;
        let ooccupied = other.pawns | other.knights | other.bishops | other.rooks | other.queens | other.kings;

        (self.pawns, self.knights, self.bishops, self.rooks, self.queens, self.kings)
            != (other.pawns, other.knights, other.bishops, other.rooks, other.queens, other.kings)
            || self.color & soccupied != other.color & ooccupied
    }
}

impl Eq for Board {}
