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

//! Move generation.
//!
//! The name `mov` was chosen because `move` is a keyword.

use crate::board::*;
use crate::piece::*;

use std::fmt;

/// The type of a move.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    Normal = 0,
    Promotion = 1,
    EnPassant = 2, // holy hell
    Castling = 3,
}

impl From<MoveType> for u8 {
    fn from(movt: MoveType) -> Self {
        movt as u8
    }
}

impl TryFrom<u8> for MoveType {
    type Error = u8;

    fn try_from(b: u8) -> Result<Self, u8> {
        Ok(match b {
            0 => MoveType::Normal,
            1 => MoveType::Promotion,
            2 => MoveType::EnPassant,
            3 => MoveType::Castling,
            _ => return Err(b),
        })
    }
}

/// An error returned by [`Move::promotion`] when the piece is not valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPromotionPiece(pub Piece);

impl fmt::Display for InvalidPromotionPiece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid promotion piece: {:?}", self.0)
    }
}

impl std::error::Error for InvalidPromotionPiece {}

/// A move.
///
/// It needs only 16 bits to be stored:
/// - bit 0-5: origin square,
/// - bit 6-11: destination square,
/// - bit 12-13: promotion piece (0 = knight, 1 = bishop, 2 = rook, 3 = queen),
/// - bit 14-15: special move flag (see [`MoveType`])
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move(u16);

impl Move {
    /// Constructs a new normal move.
    #[inline]
    pub fn new(origin: Square, dest: Square) -> Self {
        Self(
            (origin.index() as u16)
                | ((dest.index() as u16) << 6)
                | ((MoveType::Normal as u16) << 14),
        )
    }

    /// Constructs a new promotion move.
    ///
    /// Returns an `Err` if `promote` is a [`Pawn`](Piece::Pawn) or a
    /// [`King`](Piece::King).
    #[inline]
    pub fn promotion(
        origin: Square,
        dest: Square,
        promote: Piece,
    ) -> Result<Self, InvalidPromotionPiece> {
        if promote == Piece::Pawn || promote == Piece::King {
            return Err(InvalidPromotionPiece(promote));
        }

        Ok(Self(
            (origin.index() as u16)
                | ((dest.index() as u16) << 6)
                | (((promote as u16) - 1) << 12)
                | ((MoveType::Promotion as u16) << 14),
        ))
    }

    /// Costructs a new en passant.
    #[inline]
    pub fn en_passant(origin: Square, dest: Square) -> Self {
        Self(
            (origin.index() as u16)
                | ((dest.index() as u16) << 6)
                | ((MoveType::EnPassant as u16) << 14),
        )
    }

    /// Constructs a new castling move.
    ///
    /// In this situation, the origin square represents the king's position, and
    /// the destination square is the rook's position.
    #[inline]
    pub fn castling(king: Square, rook: Square) -> Self {
        Self(
            (king.index() as u16)
                | ((rook.index() as u16) << 6)
                | ((MoveType::Castling as u16) << 14),
        )
    }

    /// Returns the origin square.
    ///
    /// If the move is castling, it returns the king's position.
    #[inline]
    pub fn origin(&self) -> Square {
        Square::from_index((self.0 & 0b0011_1111) as u8)
    }

    /// Returns the destination square.
    ///
    /// If the move is castling, it returns the rook's position.
    #[inline]
    pub fn dest(&self) -> Square {
        Square::from_index(((self.0 >> 6) & 0b0011_1111) as u8)
    }

    /// Returns the promotion piece.
    pub fn promotion_piece(&self) -> Option<Piece> {
        if self.is_promotion() {
            // i love parentheses
            Some(((((self.0 >> 12) & 3) + 1) as u8).try_into().unwrap())
        } else {
            None
        }
    }

    /// Sets the origin square.
    ///
    /// If the move is castling, it sets the king's position.
    #[inline]
    pub fn set_origin(&mut self, sq: Square) {
        self.0 = (self.0 & !0b0011_1111) | (sq.index() as u16);
    }

    /// Sets the destination square.
    ///
    /// If the move is castling, it sets the rook's position.
    #[inline]
    pub fn set_dest(&mut self, sq: Square) {
        self.0 = (self.0 & !(0b0011_1111 << 6)) | ((sq.index() as u16) << 6);
    }

    /// Returns the [`MoveType`] of this move.
    pub fn move_type(&self) -> MoveType {
        (((self.0 >> 14) & 3) as u8).try_into().unwrap()
    }

    /// Checks if the move is a [`Normal`](MoveType::Normal).
    #[inline]
    pub fn is_normal(&self) -> bool {
        self.0 & (3 << 14) == (MoveType::Normal as u16) << 14
    }

    /// Checks if the move is a [`Promotion`](MoveType::Promotion).
    #[inline]
    pub fn is_promotion(&self) -> bool {
        self.0 & (3 << 14) == (MoveType::Promotion as u16) << 14
    }

    /// Checks if the move is an [`EnPassant`](MoveType::EnPassant).
    #[inline]
    pub fn is_en_passant(&self) -> bool {
        self.0 & (3 << 14) == (MoveType::EnPassant as u16) << 14
    }

    /// Checks if the move is a [`Castling`](MoveType::Castling).
    #[inline]
    pub fn is_castling(&self) -> bool {
        self.0 & (3 << 14) == (MoveType::Castling as u16) << 14
    }
}
