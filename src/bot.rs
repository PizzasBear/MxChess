use rayon::prelude::*;

use crate::{Board, Color, Move, MoveType, Piece, PieceType, Pieces};
// use std::sync::atomic::{self, AtomicI32};

pub struct Bot;

fn pieces_value(pieces: &Pieces) -> u32 {
    pieces.pawns.count_ones()
        + 3 * (pieces.knights.count_ones() + pieces.bishops.count_ones())
        + 5 * pieces.rooks.count_ones()
        + 9 * pieces.queens.count_ones()
}

impl Bot {
    fn guess_white_win(&self, board: &Board) -> i32 {
        100 * (pieces_value(&board.white_pieces) as i32 - pieces_value(&board.black_pieces) as i32)
    }

    fn eval_move(&self, mv: &Move, board: &Board, attack: u64) -> i32 {
        let mut score = 0;

        if let Some(Piece { ty, .. }) = board.get_at(1 << mv.to) {
            score += match ty {
                PieceType::King => unreachable!(),
                PieceType::Queen => 9,
                PieceType::Rook => 5,
                PieceType::Bishop => 3,
                PieceType::Knight => 3,
                PieceType::Pawn => 1,
            };
        }

        if 1 << mv.to & attack != 0 {
            score -= 9 * match mv.ty {
                MoveType::King | MoveType::Castle => unreachable!(),
                MoveType::Queen
                | MoveType::PawnQueenPromotion
                | MoveType::PawnRookPromotion
                | MoveType::PawnBishopPromotion
                | MoveType::PawnKnightPromotion => 9,
                MoveType::Rook => 5,
                MoveType::Bishop => 3,
                MoveType::Knight => 3,
                MoveType::Pawn | MoveType::PawnLeap | MoveType::PawnEnPassant => 1,
            } / 8;
        }

        score
    }

    fn eval_captures_board_rec(
        &self,
        board: &Board,
        pos: u8,
        color: Color,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        let mut moves: Vec<_> = board
            .capture_moves(color)
            .into_iter()
            .filter(|mv| mv.to == pos)
            .collect();

        if moves.is_empty() {
            if board.check_attack(color.inv()) & board.get_pieces(color).king == 0 {
                let val = self.guess_white_win(&board);
                match color {
                    Color::White => val,
                    Color::Black => -val,
                }
            } else {
                -i32::MAX
            }
        } else {
            let attack = board.check_attack(color.inv());
            moves.sort_unstable_by_key(|mv| -self.eval_move(mv, board, attack));

            let mut value = -i32::MAX;

            for mv in moves.into_iter() {
                let mut board = *board;
                board.perform_move(mv);
                value = value.max(-self.eval_captures_board_rec(
                    &board,
                    pos,
                    color.inv(),
                    -beta,
                    -alpha,
                ));
                if beta <= value {
                    return beta;
                }
                alpha = alpha.max(value);
            }

            value
        }
    }

    fn eval_board_rec(
        &self,
        board: &Board,
        color: Color,
        depth: u32,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        if depth == 0 {
            self.eval_captures_board_rec(board, board.prev_move.to, color, alpha, beta)
        } else {
            let mut moves = board.moves(color);
            if moves.is_empty() {
                if board.check_attack(color.inv()) & board.get_pieces(color).king == 0 {
                    0
                } else {
                    -i32::MAX
                }
            } else {
                let mut value = -i32::MAX;

                let attack = board.check_attack(color.inv());
                moves.sort_unstable_by_key(|mv| -self.eval_move(mv, board, attack));

                for mv in moves.into_iter() {
                    let mut board = *board;
                    board.perform_move(mv);
                    value = value.max(-self.eval_board_rec(
                        &board,
                        color.inv(),
                        depth - 1,
                        -beta,
                        -alpha,
                    ));
                    if beta <= value {
                        return beta;
                    }
                    alpha = alpha.max(value);
                }

                value
            }
        }
    }

    /// Failes if there's no legal move
    pub fn choose_move(&self, board: &Board, color: Color) -> Option<Move> {
        const DEPTH: u32 = 6;

        let mut moves = board.moves(color);

        let attack = board.check_attack(color.inv());
        moves.sort_by_key(|mv| -self.eval_move(mv, board, attack));

        moves.into_par_iter().min_by_key(|&mv| {
            let mut board = *board;
            board.perform_move(mv);
            self.eval_board_rec(&board, color.inv(), DEPTH, -i32::MAX, i32::MAX)
        })
    }
}
