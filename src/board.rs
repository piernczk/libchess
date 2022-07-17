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

use crate::piece::*;
use bitvec::prelude::*;
use itertools::izip;
use std::ops;

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

impl From<Square> for (u8, u8) {
    fn from(sq: Square) -> Self {
        (sq.file(), sq.rank())
    }
}

impl From<(u8, u8)> for Square {
    fn from(t: (u8, u8)) -> Self {
        Square::new(t.0, t.1)
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
        Self { pl: t.0, pc: t.1 }
    }
}

/// A newtype for an array of [`Bitboard`]s of all [`Piece`].
///
/// Even 2x faster than using single variables for each [`Bitboard`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pieces(pub [Bitboard; 6]);

impl Pieces {
    /// Constructs a new [`Pieces`].
    #[inline]
    pub const fn new() -> Self {
        Self([Bitboard::ZERO; 6])
    }
}

impl From<Pieces> for [Bitboard; 6] {
    fn from(pieces: Pieces) -> Self {
        pieces.0
    }
}

impl TryFrom<[Bitboard; 6]> for Pieces {
    type Error = u8;

    fn try_from(arr: [Bitboard; 6]) -> Result<Self, Self::Error> {
        // iterate through all the squares
        // i is index, p is pawn, n is knight, etc.
        for (i, p, n, b, r, q, k) in izip!(0..64, arr[0], arr[1], arr[2], arr[3], arr[4], arr[5],) {
            let bits = (p as u8)
                | ((n as u8) << 1)
                | ((b as u8) << 2)
                | ((r as u8) << 3)
                | ((q as u8) << 4)
                | ((k as u8) << 5);

            if bits.count_ones() > 1 {
                return Err(i);
            }
        }

        Ok(Self(arr))
    }
}

impl ops::Index<Piece> for Pieces {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl ops::IndexMut<Piece> for Pieces {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

/// A [`Bitboard`]-based chessboard implementation.
#[derive(Debug, Clone, Default)]
pub struct Board {
    color: Bitboard,
    pieces: Pieces,
}

impl Board {
    /// Construct an empty board.
    pub const fn new() -> Self {
        Self {
            color: Bitboard::ZERO,
            pieces: Pieces::new(),
        }
    }

    /// Check if a square is occupied by a piece.
    ///
    /// A bit faster than [`get_piece`](Self::get_piece).
    pub fn occupied(&self, sq: Square) -> bool {
        let idx = sq.index() as usize;
        self.pieces[Piece::Pawn][idx]
            | self.pieces[Piece::Knight][idx]
            | self.pieces[Piece::Bishop][idx]
            | self.pieces[Piece::Rook][idx]
            | self.pieces[Piece::Queen][idx]
            | self.pieces[Piece::King][idx]
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
        for (i, bitboard) in self.pieces.0.iter().enumerate() {
            if bitboard[sq.index() as usize] {
                return Some(Piece::try_from(i as u8).unwrap());
            }
        }

        None
    }

    /// Get the piece's type and color.
    ///
    /// A bit slower than [`get_color`](Self::get_color) and
    /// [`get_piece`](Self::get_piece), but `square_info` should be used
    /// whenever both the color and the type of a piece is needed.
    pub fn square_info(&self, sq: Square) -> Option<SquareInfo> {
        self.get_piece(sq)
            .map(|pc| SquareInfo::new(self.get_color_unchecked(sq), pc))
    }

    /// Set the color of a piece.
    #[inline]
    pub fn set_color(&mut self, sq: Square, pl: Player) {
        self.color.set(sq.index() as usize, pl.into());
    }

    /// Set the type of a piece.
    pub fn set_piece(&mut self, sq: Square, pc: Piece) {
        self.unset_square(sq);
        self.pieces[pc].set(sq.index() as usize, true);
    }

    /// Removes the piece from a square.
    pub fn unset_square(&mut self, sq: Square) {
        for bitboard in &mut self.pieces.0 {
            bitboard.set(sq.index() as usize, false);
        }
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
        if self.pieces != other.pieces {
            return false;
        }

        let soccupied = self.pieces[Piece::Pawn]
            | self.pieces[Piece::Knight]
            | self.pieces[Piece::Bishop]
            | self.pieces[Piece::Rook]
            | self.pieces[Piece::Queen]
            | self.pieces[Piece::King];

        let ooccupied = other.pieces[Piece::Pawn]
            | other.pieces[Piece::Knight]
            | other.pieces[Piece::Bishop]
            | other.pieces[Piece::Rook]
            | other.pieces[Piece::Queen]
            | other.pieces[Piece::King];

        // bitmasks (soccupied and ooccupied) are equal so if the following
        // condition is true, the boards are equal
        self.color & soccupied != other.color & ooccupied
    }
}

impl Eq for Board {}
