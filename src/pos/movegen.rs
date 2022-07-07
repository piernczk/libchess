use super::*;
use crate::board::*;
use crate::mov::*;

/// The upper bound of pseudolegal moves in any position.
///
/// See [`this topic`](https://www.stmintz.com/ccc/index.php?id=424966) on CCC.
pub const MAX_MOVES_IN_POSITION: u8 = 218;

macro_rules! gen_promotions {
    ($vec:ident, $origin:ident, $dest:ident) => {
        $vec.push(Move::promotion($origin, $dest, Piece::Knight).unwrap());
        $vec.push(Move::promotion($origin, $dest, Piece::Bishop).unwrap());
        $vec.push(Move::promotion($origin, $dest, Piece::Rook).unwrap());
        $vec.push(Move::promotion($origin, $dest, Piece::Queen).unwrap());
    };
}

#[allow(dead_code)]
pub fn movegen_pseudo_legal(pos: &Position) -> Vec<Move> {
    let mut result = Vec::with_capacity(MAX_MOVES_IN_POSITION as usize);
    for sq in (0..64).map(Square::from_index) {
        let (file, rank) = sq.into();
        let (pl, pc) = match pos.get_board().square_info(sq) {
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
                    if !pos.get_board().occupied(sq_front) {
                        gen_promotions!(result, sq, sq_front);
                    }

                    if let Some(sq_west_front) = sq_west_front {
                        if pos.get_board().occupied(sq_west_front) {
                            // "if you need more than 3 levels of indentation,
                            // you're screwed anyway, and should fix your
                            // program" haha sure
                            gen_promotions!(result, sq, sq_west_front);
                        }
                    }

                    if let Some(sq_east_front) = sq_east_front {
                        if pos.get_board().occupied(sq_east_front) {
                            gen_promotions!(result, sq, sq_east_front);
                        }
                    }
                } else {
                    if let Some(sq_west_front) = sq_west_front {
                        if pos.get_board().occupied(sq_west_front)
                            || pos.get_en_passant_square() == Some(sq_west_front)
                        {
                            result.push(Move::new(sq, sq_west_front));
                        }
                    }

                    if let Some(sq_east_front) = sq_east_front {
                        if pos.get_board().occupied(sq_east_front)
                            || pos.get_en_passant_square() == Some(sq_east_front)
                        {
                            result.push(Move::new(sq, sq_east_front));
                        }
                    }

                    // i feel like i could have made these 30 lines above
                    // disappear using a macro
                    // but i don't have an idea for the name of said macro

                    if !pos.get_board().occupied(sq_front) {
                        result.push(Move::new(sq, sq_front));

                        let sq_front_front: Square;
                        if rank == 1 && pl == Player::White {
                            sq_front_front = (file, rank + 2).into();
                        } else if rank == 6 && pl == Player::Black {
                            sq_front_front = (file, rank - 2).into();
                        } else {
                            continue;
                        }

                        if !pos.get_board().occupied(sq_front_front) {
                            result.push(Move::new(sq, sq_front_front));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    result
}
