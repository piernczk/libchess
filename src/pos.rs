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

//! Position-related things.
use bitflags::bitflags;

use crate::board::*;
use crate::piece::*;

bitflags! {
    /// Castling availability in a position.
    pub struct CastlingAvailability: u8 {
        const BLACK_QUEENSIDE = 0b0001;
        const WHITE_QUEENSIDE = 0b0010;
        const BLACK_KINGSIDE = 0b0100;
        const WHITE_KINGSIDE = 0b1000;

        const BLACK = Self::BLACK_QUEENSIDE.bits | Self::BLACK_KINGSIDE.bits;
        const WHITE = Self::WHITE_QUEENSIDE.bits | Self::WHITE_KINGSIDE.bits;

        const NONE = 0b0000;
        const ALL = 0b1111;
    }
}

impl CastlingAvailability {
    pub fn kingside(pl: Player) -> Self {
        match pl {
            Player::White => Self::WHITE_KINGSIDE,
            Player::Black => Self::BLACK_KINGSIDE,
        }
    }

    pub fn queenside(pl: Player) -> Self {
        match pl {
            Player::White => Self::WHITE_QUEENSIDE,
            Player::Black => Self::BLACK_QUEENSIDE,
        }
    }

    pub fn both(pl: Player) -> Self {
        match pl {
            Player::White => Self::WHITE,
            Player::Black => Self::BLACK,
        }
    }
}

/// A position, as understood in Forsyth-Edwards Notation.
///
/// A position consists of:
/// - piece placement ([`Board`])
/// - active color / turn ([`Player`])
/// - castling availability ([`CastlingAvailability`])
/// - en passant square ([`Square`])
/// - halfmove clock ([`u8`])
/// - fullmove number ([`u16`])
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    board: Board,
    turn: Player,
    castling_availability: CastlingAvailability,
    en_passant: Option<Square>,
    halfmove_clock: u8,
    fullmove_number: u16,
}

impl Position {
    /// Constructs an empty position.
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            turn: Player::White,
            castling_availability: CastlingAvailability::ALL,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Returns a reference to the [`Board`].
    #[inline]
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Returns the turn.
    #[inline]
    pub fn get_turn(&self) -> Player {
        self.turn
    }

    /// Returns the players' castling availability.
    #[inline]
    pub fn get_castling_availability(&self) -> CastlingAvailability {
        self.castling_availability
    }

    /// Returns the en passant square.
    ///
    /// It's the square that a pawn "jumps over" after a double push. It's
    /// set even if en passant is not possible in the position. It's `None` if
    /// the last move wasn't a double pawn push.
    #[inline]
    pub fn get_en_passant_square(&self) -> Option<Square> {
        self.en_passant
    }

    /// Returns the number of halfmoves since the last capture or pawn push.
    #[inline]
    pub fn get_halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    /// Returns the number of full moves since the beginning of the game.
    #[inline]
    pub fn get_fullmove_number(&self) -> u16 {
        self.fullmove_number
    }
}
