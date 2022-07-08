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

//! Pieces-related things.

/// White or black pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl Player {
    /// Returns `Black` if it's `White` and `White` otherwise.
    pub fn opponent(self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

impl From<Player> for u8 {
    fn from(pl: Player) -> u8 {
        pl as u8
    }
}

impl TryFrom<u8> for Player {
    type Error = ();

    fn try_from(b: u8) -> Result<Self, ()> {
        Ok(match b {
            0 => Self::White,
            1 => Self::Black,
            _ => return Err(()),
        })
    }
}

impl From<Player> for bool {
    fn from(pl: Player) -> bool {
        pl == Player::Black
    }
}

impl From<bool> for Player {
    fn from(b: bool) -> Self {
        if b {
            Self::Black
        } else {
            Self::White
        }
    }
}

/// Chess piece types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl From<Piece> for u8 {
    fn from(p: Piece) -> Self {
        p as u8
    }
}

impl TryFrom<u8> for Piece {
    type Error = ();

    fn try_from(b: u8) -> Result<Self, ()> {
        Ok(match b {
            0 => Self::Pawn,
            1 => Self::Knight,
            2 => Self::Bishop,
            3 => Self::Rook,
            4 => Self::Queen,
            5 => Self::King,
            _ => return Err(()),
        })
    }
}
