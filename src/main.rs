// The Octal Chess Board:
//
// 8 | 70  71  72  73  74  75  76  77
//   |
// 7 | 60  61  62  63  64  65  66  67
//   |
// 6 | 50  51  52  53  54  55  56  57
//   |
// 5 | 40  41  42  43  44  45  46  47
//   |
// 4 | 30  31  32  33  34  35  36  37
//   |
// 3 | 20  21  22  23  24  25  26  27
//   |
// 2 | 10  11  12  13  14  15  16  17
//   |
// 1 | 00  01  02  03  04  05  06  07
//   +-------------------------------
//      a   b   c   d   e   f   g   h

// This is because labled block are still unreleased and are immitated with never looping loops.
#![allow(clippy::never_loop)]

pub mod bit_iter;
pub mod board;

pub use bit_iter::BitIterator;
pub use board::{print_moves, Board, Color, Move, MoveType, Piece, PieceType};

use std::io::{self, Write};

pub fn chess_pos(chs: &[u8]) -> u8 {
    assert_eq!(chs.len(), 2);
    assert!((b'a'..=b'h').contains(&chs[0]));
    assert!((b'1'..=b'8').contains(&chs[1]));
    8 * (chs[1] - b'1') + (chs[0] - b'a')
}

fn to_chess_pos(x: u8) -> String {
    String::from_utf8([b'a' + (x & 7), b'1' + x / 8].to_vec()).unwrap()
}

pub fn demo() {
    let moves: &[_] = &[
        Move {
            from: chess_pos(b"e2"),
            to: chess_pos(b"e4"),
            ty: MoveType::PawnLeap,
        },
        Move {
            from: chess_pos(b"d7"),
            to: chess_pos(b"d5"),
            ty: MoveType::PawnLeap,
        },
        Move {
            from: chess_pos(b"e4"),
            to: chess_pos(b"e5"),
            ty: MoveType::Pawn,
        },
        Move {
            from: chess_pos(b"f7"),
            to: chess_pos(b"f5"),
            ty: MoveType::PawnLeap,
        },
        Move {
            from: chess_pos(b"e5"),
            to: chess_pos(b"f6"),
            ty: MoveType::PawnEnPassant,
        },
        Move {
            from: chess_pos(b"g8"),
            to: chess_pos(b"f6"),
            ty: MoveType::Knight,
        },
        Move {
            from: chess_pos(b"f1"),
            to: chess_pos(b"b5"),
            ty: MoveType::Bishop,
        },
        Move {
            from: chess_pos(b"c7"),
            to: chess_pos(b"c6"),
            ty: MoveType::Pawn,
        },
        Move {
            from: chess_pos(b"g1"),
            to: chess_pos(b"h3"),
            ty: MoveType::Knight,
        },
        Move {
            from: chess_pos(b"c6"),
            to: chess_pos(b"b5"),
            ty: MoveType::Pawn,
        },
        Move {
            from: chess_pos(b"e1"),
            to: chess_pos(b"g1"),
            ty: MoveType::Castle,
        },
    ];

    let mut board = Board::new();

    let mut color = Color::White;
    for &mv in moves.iter() {
        board.print(color);
        print_moves(&board, &board.moves(color));
        println!("attack: 0x{:x}", board.check_attack(color));
        println!();
        assert!(board
            .moves(color)
            .iter()
            .all(|&mv| board.is_legal(color, mv)));
        board.perform_move(mv);

        color = color.inv();
    }

    board.print(color);
    print_moves(&board, &board.moves(color));
    println!("attack: 0x{:x}", board.check_attack(color));
    println!();
}

pub fn two_player_mode() -> io::Result<()> {
    let mut buf = String::new();

    let mut board = Board::new();

    println!("Move format: \"<Initial chess position> <Target chess position>\"");
    println!("  castling will be inferred from the king's move");
    println!("  for example: g8 f6");
    let mut color = Color::White;
    loop {
        println!();
        match color {
            Color::White => println!("White's move:"),
            Color::Black => println!("Black's move:"),
        }
        println!("------------");

        let moves = board.moves(color);
        if moves.is_empty() {
            println!("YOU LOST")
        }
        board.print(color);
        print_moves(&board, &moves);

        buf.clear();
        print!("Your move: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut buf)?;

        let (from, to) = {
            let mut iter = buf.trim().split(' ');
            let from = chess_pos(iter.next().expect("Bad input").as_bytes());
            let to = chess_pos(iter.next().expect("Bad input").as_bytes());
            assert!(iter.next().is_none(), "Bad input");

            (from, to)
        };

        let mut mv = board
            .get_legal_move(color, from, to)
            .expect("This move is illegal");
        if mv.ty == MoveType::PawnQueenPromotion {
            print!("Choose pawn promotion (q,r,b,n): ");
            io::stdout().flush()?;

            buf.clear();
            io::stdin().read_line(&mut buf)?;
            buf.make_ascii_lowercase();

            mv.ty = match buf.as_str().trim() {
                "q" | "queen" => MoveType::PawnQueenPromotion,
                "r" | "rook" => MoveType::PawnRookPromotion,
                "b" | "bishop" => MoveType::PawnBishopPromotion,
                "n" | "knight" => MoveType::PawnKnightPromotion,
                _ => panic!("Bad promotion path"),
            };
        }
        board.perform_move(mv);

        color = color.inv();
    }

    // Ok(())
}

fn main() -> io::Result<()> {
    Ok(())
}
