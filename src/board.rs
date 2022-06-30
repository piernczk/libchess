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

//! Board-related things.

use bitvec::prelude::*;
use crate::piece::*;

/// 64-bit bitboard.
///
/// A bitboard is used to represent single feature with one bit for each square.
/// For example, it may be used to save presence of a type of a piece or its
/// color. It also can be used in the evaluation process, for example as magic
/// bitboards or pawn hash tables.
pub type Bitboard = BitArr!(for 64);

/// Single square on a board.
///
/// Basically a wrapped `u8` (called the index) whose 3 LSB represent the file
/// and 3 higher bits represent the rank (both in the exclusive range `[0-8)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    /// Constructs a new [`Square`] from its file and rank.
    ///
    /// # Panics
    ///
    /// Panics when `file >= 8 || rank >= 8`.
    #[inline]
    pub const fn new(file: u8, rank: u8) -> Self {
        assert!(file < 8);
        assert!(rank < 8);
        Self(file | (rank << 3))
    }

    /// Constructs a new [`Square`] from its index on a chessboard.
    ///
    /// # Panics
    ///
    /// Panics when `idx >= 64`.
    #[inline]
    pub const fn from_index(idx: u8) -> Self {
        assert!(idx < 64);
        Self(idx)
    }

    /// Returns the index of this square on a chessboard.
    #[inline]
    pub fn index(self) -> u8 {
        self.0
    }

    /// Returns the file of this square.
    #[inline]
    pub fn file(self) -> u8 {
        self.0 & 0b0000_0111
    }

    /// Returns the rank of this square.
    #[inline]
    pub fn rank(self) -> u8 {
        self.0 >> 3
    }

    /// Sets the index of this square to a new one.
    ///
    /// # Panics
    ///
    /// Panics if `idx >= 64`.
    #[inline]
    pub fn set_index(&mut self, idx: u8) {
        assert!(idx < 64);
        self.0 = idx;
    }

    /// Sets the file of this square.
    ///
    /// # Panics
    ///
    /// Panics if `file >= 8`.
    #[inline]
    pub fn set_file(&mut self, file: u8) {
        assert!(file < 8);
        self.0 &= 0b0011_1000 | file;
    }

    /// Sets the rank of this square.
    ///
    /// # Panics
    ///
    /// Panics if `rank >= 8`.
    #[inline]
    pub fn set_rank(&mut self, rank: u8) {
        assert!(rank < 8);
        self.0 &= 0b0000_0111 | (rank << 3);
    }
}

/// A pair of values returned by [`Square::square_info`].
///
/// They represent the color of the piece and the piece itself, always in that
/// order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SquareInfo {
    pub pl: Player,
    pub pc: Piece,
}

impl SquareInfo {
    /// Constructs a new [`SquareInfo`].
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

/// A [`Bitboard`]-based chessboard implementation.
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
    /// Construct an empty board.
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

    /// Check if a square is occupied by a piece.
    ///
    /// A bit faster than [`get_piece`](Self::get_piece).
    pub fn occupied(&self, sq: Square) -> bool {
        let idx = sq.index() as usize;

        self.pawns[idx]
            || self.knights[idx]
            || self.bishops[idx]
            || self.rooks[idx]
            || self.queens[idx]
            || self.kings[idx]
    }

    /// Get the color of the piece on a square.
    ///
    /// [`square_info`](Self::square_info) should be used unless the piece type
    /// itself is not needed for sure, as `get_color` is only a bit faster.
    pub fn get_color(&self, sq: Square) -> Option<Player> {
        if self.occupied(sq) {
            Some(self.get_color_unchecked(sq))
        } else {
            None
        }
    }

    /// Get the color of the piece on a square if it's there.
    ///
    /// The result is undefined if there is no piece on that square, although
    /// it's a lot faster.
    #[inline]
    pub fn get_color_unchecked(&self, sq: Square) -> Player {
        self.color[sq.index() as usize].into()
    }

    /// Get the piece type on a square.
    ///
    /// [`square_info`](Self::square_info) should be used unless the piece color
    /// is not needed for sure, as `get_piece` is only a bit faster.
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

    /// Get the piece's type and color.
    ///
    /// A bit slower than [`get_color`](Self::get_color) and
    /// [`get_piece`](Self::get_piece), but `square_info` should be used
    /// whenever both the color and the type of a piece is needed.
    pub fn square_info(&self, sq: Square) -> Option<SquareInfo> {
        match self.get_piece(sq) {
            Some(pc) => Some(SquareInfo::new(self.get_color_unchecked(sq), pc)),
            None => None
        }
    }

    /// Set the color of a piece.
    #[inline]
    pub fn set_color(&mut self, sq: Square, pl: Player) {
        self.color.set(sq.index() as usize, pl.into());
    }

    /// Set the type of a piece.
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

    /// Removes the piece from a square.
    pub fn unset_square(&mut self, sq: Square) {
        let idx = sq.index() as usize;

        self.pawns.set(idx, false);
        self.knights.set(idx, false);
        self.bishops.set(idx, false);
        self.rooks.set(idx, false);
        self.queens.set(idx, false);
        self.kings.set(idx, false);
    }

    /// Sets the color and the type of a piece from a [`SquareInfo`].
    #[inline]
    pub fn set_square_info(&mut self, sq: Square, sqi: SquareInfo) {
        self.set_color(sq, sqi.pl);
        self.set_piece(sq, sqi.pc);
    }

    /// Sets the color and the type of a piece.
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

        // can't compare soccupied with ooccupied as e.g. a knight could be
        // replaced a rook in one board and they would be recognized as equal.
        (self.pawns, self.knights, self.bishops, self.rooks, self.queens, self.kings)
            != (other.pawns, other.knights, other.bishops, other.rooks, other.queens, other.kings)
        // bitmasks (soccupied and ooccupied) are equal so if the following
        // condition is true, the boards are equal
            || self.color & soccupied != other.color & ooccupied
    }
}

impl Eq for Board {}
