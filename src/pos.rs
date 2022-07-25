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

mod movegen;

use bitflags::bitflags;

use crate::board::*;
use crate::mov::*;
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
    /// Kingside castling of a player.
    pub fn kingside(pl: Player) -> Self {
        match pl {
            Player::White => Self::WHITE_KINGSIDE,
            Player::Black => Self::BLACK_KINGSIDE,
        }
    }

    /// Queenside castling of a player.
    pub fn queenside(pl: Player) -> Self {
        match pl {
            Player::White => Self::WHITE_QUEENSIDE,
            Player::Black => Self::BLACK_QUEENSIDE,
        }
    }

    /// Both castling sides of a player.
    pub fn both(pl: Player) -> Self {
        match pl {
            Player::White => Self::WHITE,
            Player::Black => Self::BLACK,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moves {
    pub moves: Vec<Move>,
    pub white_checked: bool,
    pub black_checked: bool,
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

    /// The position of a player's king.
    pub fn king_pos(&self, pl: Player) -> Result<Square, ()> {
        let sqi = Some((pl, Piece::King).into());
        for i in 0..64 {
            let sq = Square::from_index(i);
            if self.get_board().square_info(sq) == sqi {
                return Ok(sq);
            }
        }

        Err(())
    }

    /// Generates all pseudolegal moves available in the position.
    ///
    /// Also returns whether or not the players' kings are in check. It's
    /// checked because you need the moves list to check it anyway, and there is
    /// no caching. Although, it's useful for determining if the moves are
    /// actually legal. If you don't want it, see
    /// [`movegen_pseudo_legal_ncc`](Self::movegen_pseudo_legal_ncc).
    pub fn movegen_pseudo_legal(&self) -> Moves {
        let moves = self.movegen_pseudo_legal_ncc();
        let white_king = self.king_pos(Player::White);
        let black_king = self.king_pos(Player::Black);

        let mut white_checked = false;
        let mut black_checked = false;
        for mov in &moves {
            if Ok(mov.dest()) == white_king {
                white_checked = true;
            } else if Ok(mov.dest()) == black_king {
                black_checked = true;
            }
        }

        Moves { moves, white_checked, black_checked }
    }

    /// Generates all pseudolegal moves available in a position.
    ///
    /// Doesn't check if the player's kings are in check (ncc - no check check).
    pub fn movegen_pseudo_legal_ncc(&self) -> Vec<Move> {
        movegen::movegen_pseudo_legal(&self)
    }

    /// Make a move **without** checking its legality.
    ///
    /// Because of that, if you give an impossible move like f3-b3 en passant,
    /// weird stuff can happen. You have to generate and possibly cache the
    /// legal moves first to not break the position.
    pub fn make_move(&mut self, mov: Move) {
        self.en_passant = None;
        match mov.move_type() {
            MoveType::Normal => {
                let origin = mov.origin();
                let dest = mov.dest();

                match self.board.square_info(origin) {
                    Some(origini) => {
                        if origini.pc == Piece::Pawn {
                            if origini.pl == Player::White && origin.rank() == 1 && dest.rank() == 3 {
                                // x2-x4 with a white pawn
                                self.en_passant = Some((origin.file(), 2).into());
                            } else if origini.pl == Player::Black && origin.rank() == 6 && dest.rank() == 4 {
                                // x7-x5 with a black pawn
                                self.en_passant = Some((origin.file(), 5).into());
                            }
                            self.halfmove_clock = 0; // a pawn push always resets
                        } else if self.board.occupied(dest) { // if it was a pawn it was reset already
                            self.halfmove_clock = 0;
                        } else {
                            self.halfmove_clock += 1;
                        }

                        self.board.unset_square(origin);
                        self.board.set_square_info(dest, origini);
                    }
                    None => self.board.unset_square(dest), // moving an empty square
                }
            }
            MoveType::EnPassant => {
                let origin = mov.origin();
                let dest = mov.dest();

                match self.board.square_info(origin) {
                    Some(origini) => {
                        self.board.unset_square(origin);
                        self.board.set_square_info(dest, origini);
                    }
                    None => self.board.unset_square(dest),
                }

                if dest.rank() > origin.rank() { // white en passant
                    self.board.unset_square((dest.file(), dest.rank() - 1).into());
                } else { // black en passant
                    self.board.unset_square((dest.file(), dest.rank() + 1).into());
                }

                self.halfmove_clock = 0; // pawn move
            }
            MoveType::Promotion => {
                let origin = mov.origin();
                let dest = mov.dest();
                let pl = self.board.get_color_unchecked(origin);

                self.board.unset_square(origin);
                self.board.set_square(dest, pl, mov.promotion_piece().unwrap());
                self.halfmove_clock = 0;
            }
            MoveType::Castling => {
                let king = mov.origin();
                let king_rank = king.rank();
                let rook = mov.dest();
                let pl = self.board.get_color_unchecked(king);

                self.board.unset_square(king);
                self.board.unset_square(rook);

                if king.file() > rook.file() { // queenside
                    self.board.set_square((king_rank, 2).into(), pl, Piece::King);
                    self.board.set_square((king_rank, 3).into(), pl, Piece::Rook);
                } else { // kingside
                    self.board.set_square((king_rank, 6).into(), pl, Piece::King);
                    self.board.set_square((king_rank, 5).into(), pl, Piece::Rook);
                }

                self.castling_availability &= !CastlingAvailability::both(self.turn);
                self.halfmove_clock += 1; // for some reason
            }
        }

        self.turn = match self.turn {
            Player::White => Player::Black,
            Player::Black => {
                self.fullmove_number += 1;
                Player::White
            }
        };
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
