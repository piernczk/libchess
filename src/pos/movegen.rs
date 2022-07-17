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

use super::*;
use crate::board::*;
use crate::mov::*;

/// The upper bound of pseudolegal moves in any position.
///
/// See [`this topic`](https://www.stmintz.com/ccc/index.php?id=424966) on CCC.
pub const MAX_MOVES_IN_POSITION: u8 = 218;

macro_rules! gen_promotions {
    ($vec:ident, $origin:expr, $dest:expr) => {
        $vec.push(Move::promotion($origin, $dest, Piece::Knight).unwrap());
        $vec.push(Move::promotion($origin, $dest, Piece::Bishop).unwrap());
        $vec.push(Move::promotion($origin, $dest, Piece::Rook).unwrap());
        $vec.push(Move::promotion($origin, $dest, Piece::Queen).unwrap());
    };
}

macro_rules! found_a_name {
    ($sq:ident, $cond:expr => $e:block) => {
        if let Some($sq) = $sq {
            if $cond {
                $e
            }
        }
    };
}

macro_rules! move_not_capturing_self {
    ($vec:ident, $b:ident, $origin:expr, $dest:expr, $pl:expr) => {
        if $b.get_color($dest) != Some($pl) {
            $vec.push(Move::new($origin, $dest));
        }
    };
}

macro_rules! linegen_loop {
    ($vec:ident, $b:ident, $sq:expr, $pl:expr; $range:expr, $i:ident => $dest:expr) => {
        for $i in $range {
            let dest = $dest;
            if let Some(c) = $b.get_color(dest) {
                if c != $pl {
                    $vec.push(Move::new($sq, dest));
                }
                break;
            } else {
                $vec.push(Move::new($sq, dest));
            }
        }
    };
}

fn diagonals(vec: &mut Vec<Move>, b: &Board, sq: Square, pl: Player) {
    let file = sq.file();
    let rank = sq.rank();

    // south west
    linegen_loop!(vec, b, sq, pl; 0..file.min(rank), i => (file - i, rank - i).into());

    // south east
    linegen_loop!(vec, b, sq, pl; 0..(8 - file).min(rank), i => (file + i, rank - i).into());

    // north east
    linegen_loop!(vec, b, sq, pl; 0..(8 - file.max(rank)), i => (file + i, rank + i).into());

    // north west
    linegen_loop!(vec, b, sq, pl; 0..file.min(8 - rank), i => (file - i, rank + i).into());
}

fn horvert(vec: &mut Vec<Move>, b: &Board, sq: Square, pl: Player) {
    let file = sq.file();
    let rank = sq.rank();

    // west
    linegen_loop!(vec, b, sq, pl; (0..file).rev(), i => (i, rank).into());

    // east
    linegen_loop!(vec, b, sq, pl; (file + 1)..8, i => (i, rank).into());

    // south
    linegen_loop!(vec, b, sq, pl; (0..rank).rev(), i => (file, i).into());

    // north
    linegen_loop!(vec, b, sq, pl; (rank + 1)..8, i => (file, i).into());
}

#[allow(dead_code)]
pub fn movegen_pseudo_legal(pos: &Position) -> Vec<Move> {
    let b = pos.get_board();
    let mut result = Vec::with_capacity(MAX_MOVES_IN_POSITION as usize);

    for sq in (0..64).map(Square::from_index) {
        let (file, rank) = sq.into();
        let (pl, pc) = match b.square_info(sq) {
            Some(sqi) => sqi.into(),
            None => continue,
        };

        match pc {
            Piece::Pawn => {
                let sq_front: Square;
                let mut sq_west_front: Option<Square> = None;
                let mut sq_east_front: Option<Square> = None;

                if rank < 7 && pl == Player::White {
                    sq_front = (file, rank + 1).into();
                    if file > 0 {
                        sq_west_front = Some((file - 1, rank + 1).into());
                    }
                    if file < 7 {
                        sq_east_front = Some((file + 1, rank + 1).into());
                    }
                } else if rank > 0 && pl == Player::Black {
                    sq_front = (file, rank - 1).into();
                    if file > 0 {
                        sq_west_front = Some((file - 1, rank - 1).into());
                    }
                    if file < 7 {
                        sq_east_front = Some((file + 1, rank - 1).into());
                    }
                } else {
                    continue;
                }

                // make these immutable
                let sq_west_front = sq_west_front;
                let sq_east_front = sq_east_front;

                if (rank == 6 && pl == Player::White) || (rank == 1 && pl == Player::Black) {
                    // standard promotion
                    if !b.occupied(sq_front) {
                        gen_promotions!(result, sq, sq_front);
                    }

                    // capture and promotion
                    found_a_name!(sq_west_front, b.occupied(sq_west_front) => {
                        gen_promotions!(result, sq, sq_west_front);
                    });
                    found_a_name!(sq_east_front, b.occupied(sq_east_front) => {
                        gen_promotions!(result, sq, sq_east_front);
                    });
                } else {
                    // standard capture
                    found_a_name!(sq_west_front, b.occupied(sq_west_front)
                        || pos.get_en_passant_square() == Some(sq_west_front) =>
                    {
                        gen_promotions!(result, sq, sq_west_front);
                    });
                    found_a_name!(sq_east_front, b.occupied(sq_east_front)
                        || pos.get_en_passant_square() == Some(sq_east_front) =>
                    {
                        gen_promotions!(result, sq, sq_east_front);
                    });

                    // standard pawn push
                    if !b.occupied(sq_front) {
                        result.push(Move::new(sq, sq_front));

                        let sq_front_front: Square;
                        if rank == 1 && pl == Player::White {
                            // "if you need more than 3 levels of indentation,
                            // you're screwed anyway, and should fix your
                            // program" haha sure
                            sq_front_front = (file, rank + 2).into();
                        } else if rank == 6 && pl == Player::Black {
                            sq_front_front = (file, rank - 2).into();
                        } else {
                            continue;
                        }

                        // double pawn push
                        if !b.occupied(sq_front_front) {
                            result.push(Move::new(sq, sq_front_front));
                        }
                    }
                }
            }
            Piece::Knight => {
                // north
                if rank < 7 {
                    // north east east
                    if file < 6 {
                        move_not_capturing_self!(result, b, sq, (file + 2, rank + 1).into(), pl);
                    }
                    // north west west
                    if file > 1 {
                        move_not_capturing_self!(result, b, sq, (file - 2, rank + 1).into(), pl);
                    }

                    if rank < 6 {
                        // north north east
                        if file < 7 {
                            move_not_capturing_self!(
                                result,
                                b,
                                sq,
                                (file + 1, rank + 2).into(),
                                pl
                            );
                        }
                        // north north west
                        if file > 0 {
                            move_not_capturing_self!(
                                result,
                                b,
                                sq,
                                (file - 1, rank + 2).into(),
                                pl
                            );
                        }
                    }
                }
                // south
                if rank > 0 {
                    // south east east
                    if file < 6 {
                        move_not_capturing_self!(result, b, sq, (file + 2, rank - 1).into(), pl);
                    }
                    // south west west
                    if file > 1 {
                        move_not_capturing_self!(result, b, sq, (file - 2, rank - 1).into(), pl);
                    }

                    if rank > 1 {
                        // south south east
                        if file < 7 {
                            move_not_capturing_self!(
                                result,
                                b,
                                sq,
                                (file + 1, rank - 2).into(),
                                pl
                            );
                        }
                        // south south west
                        if file > 0 {
                            move_not_capturing_self!(
                                result,
                                b,
                                sq,
                                (file - 1, rank - 2).into(),
                                pl
                            );
                        }
                    }
                }
            }
            Piece::Bishop => diagonals(&mut result, &b, sq, pl),
            Piece::Rook => horvert(&mut result, &b, sq, pl),
            Piece::Queen => {
                diagonals(&mut result, &b, sq, pl);
                horvert(&mut result, &b, sq, pl);
            }
            Piece::King => {
                if rank > 0 {
                    move_not_capturing_self!(result, b, sq, (file, rank - 1).into(), pl);
                }

                if rank < 7 {
                    move_not_capturing_self!(result, b, sq, (file, rank + 1).into(), pl);
                }

                if file > 0 {
                    move_not_capturing_self!(result, b, sq, (file - 1, rank).into(), pl);

                    if rank > 0 {
                        move_not_capturing_self!(result, b, sq, (file - 1, rank - 1).into(), pl);
                    }

                    if rank < 7 {
                        move_not_capturing_self!(result, b, sq, (file - 1, rank + 1).into(), pl);
                    }
                }

                if file < 7 {
                    move_not_capturing_self!(result, b, sq, (file + 1, rank).into(), pl);

                    if rank > 0 {
                        move_not_capturing_self!(result, b, sq, (file + 1, rank - 1).into(), pl);
                    }

                    if rank < 7 {
                        move_not_capturing_self!(result, b, sq, (file + 1, rank + 1).into(), pl);
                    }
                }

                if pos
                    .get_castling_availability()
                    .intersects(CastlingAvailability::kingside(pl))
                    && !b.occupied((file + 1, rank).into())
                    && !b.occupied((file + 2, rank).into())
                {
                    result.push(Move::castling(sq, Square::new(7, rank)));
                }

                if pos
                    .get_castling_availability()
                    .intersects(CastlingAvailability::queenside(pl))
                    && !b.occupied((file - 1, rank).into())
                    && !b.occupied((file - 2, rank).into())
                    && !b.occupied((file - 3, rank).into())
                {
                    result.push(Move::castling(sq, Square::new(0, rank)));
                }
            }
        }
    }

    result.shrink_to_fit();
    result
}
