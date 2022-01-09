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

use bitflags::bitflags;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum MoveType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,

    Pawn,
    PawnLeap,
    PawnEnPassant,

    PawnQueenPromotion,
    PawnRookPromotion,
    PawnBishopPromotion,
    PawnKnightPromotion,

    Castle,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum PieceType {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Piece {
    color: Color,
    ty: PieceType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub struct Pieces {
    pub all: u64,
    pub king: u64,
    pub queens: u64,
    pub rooks: u64,
    pub bishops: u64,
    pub knights: u64,
    pub pawns: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub struct Board {
    pub white_pieces: Pieces,
    pub black_pieces: Pieces,
    pub prev_move: Move,
    flags: ChessFlags,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C, align(32))]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub ty: MoveType,
}

bitflags! {
    pub struct ChessFlags: u8 {
        const WHITE_KINGS_CASTLE  = 0b0001;
        const WHITE_QUEENS_CASTLE = 0b0010;
        const BLACK_KINGS_CASTLE  = 0b0100;
        const BLACK_QUEENS_CASTLE = 0b1000;

        const INIT =
            Self::WHITE_KINGS_CASTLE.bits
            | Self::WHITE_QUEENS_CASTLE.bits
            | Self::BLACK_KINGS_CASTLE.bits
            | Self::BLACK_QUEENS_CASTLE.bits;
    }
}

impl Color {
    #[inline]
    pub fn inv(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl From<u8> for PieceType {
    #[inline]
    fn from(n: u8) -> Self {
        match n {
            0 => Self::King,
            1 => Self::Queen,
            2 => Self::Rook,
            3 => Self::Bishop,
            4 => Self::Knight,
            5 => Self::Pawn,
            _ => panic!("Failed to convert u8 to PieceType because it was out of the range 0..6"),
        }
    }
}

impl Pieces {
    // TODO: Improve the `get()` & `get_mut()` methods for release.
    //       Pointer arithmetic is ugly and it depends on the field order of `Self`
    //       and the `PieceType` enum variant values.

    #[inline]
    pub fn get(&self, ty: PieceType) -> u64 {
        if cfg!(debug_assertions) {
            match ty {
                PieceType::King => self.king,
                PieceType::Queen => self.queens,
                PieceType::Rook => self.rooks,
                PieceType::Bishop => self.bishops,
                PieceType::Knight => self.knights,
                PieceType::Pawn => self.pawns,
            }
        } else {
            unsafe { *(&self.king as *const u64).add(ty as _) }
        }
    }

    #[inline]
    pub fn get_mut(&mut self, ty: PieceType) -> &mut u64 {
        if cfg!(debug_assertions) {
            match ty {
                PieceType::King => &mut self.king,
                PieceType::Queen => &mut self.queens,
                PieceType::Rook => &mut self.rooks,
                PieceType::Bishop => &mut self.bishops,
                PieceType::Knight => &mut self.knights,
                PieceType::Pawn => &mut self.pawns,
            }
        } else {
            unsafe { &mut *(&mut self.king as *mut u64).add(ty as _) }
        }
    }

    pub fn get_at(&self, bit_pos: u64) -> Option<PieceType> {
        if self.all & bit_pos != 0 {
            Some(PieceType::from(
                PieceType::King as u8 & 0u8.wrapping_sub((self.king & bit_pos != 0) as _)
                    | PieceType::Queen as u8 & 0u8.wrapping_sub((self.queens & bit_pos != 0) as _)
                    | PieceType::Rook as u8 & 0u8.wrapping_sub((self.rooks & bit_pos != 0) as _)
                    | PieceType::Bishop as u8
                        & 0u8.wrapping_sub((self.bishops & bit_pos != 0) as _)
                    | PieceType::Knight as u8
                        & 0u8.wrapping_sub((self.knights & bit_pos != 0) as _)
                    | PieceType::Pawn as u8 & 0u8.wrapping_sub((self.pawns & bit_pos != 0) as _),
            ))
        } else {
            None
        }
    }

    /// It's faster than clear if you know that it's gonna clear something,
    /// otherwise it's slower.
    pub fn clear_unchecked(&mut self, bit_pos: u64) {
        self.all &= !bit_pos;
        self.king &= !bit_pos;
        self.queens &= !bit_pos;
        self.rooks &= !bit_pos;
        self.bishops &= !bit_pos;
        self.knights &= !bit_pos;
        self.pawns &= !bit_pos;
    }

    pub fn clear(&mut self, bit_pos: u64) -> bool {
        if self.all & bit_pos != 0 {
            self.clear_unchecked(bit_pos);
            true
        } else {
            false
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            white_pieces: Pieces {
                pawns: 0xff00,
                rooks: 0x0081,
                knights: 0x0042,
                bishops: 0x0024,
                queens: 0x0008,
                king: 0x0010,
                all: 0xffff,
            },
            black_pieces: Pieces {
                pawns: 0x00ff_0000_0000_0000,
                rooks: 0x8100_0000_0000_0000,
                knights: 0x4200_0000_0000_0000,
                bishops: 0x2400_0000_0000_0000,
                queens: 0x800_0000_0000_0000,
                king: 0x1000_0000_0000_0000,
                all: 0xffff_0000_0000_0000,
            },
            prev_move: Move {
                from: 0o74,
                to: 0o74,
                ty: MoveType::King,
            },
            flags: ChessFlags::INIT,
        }
    }

    pub fn get_at(&self, bit_pos: u64) -> Option<Piece> {
        self.white_pieces
            .get_at(bit_pos)
            .map(|ty| Piece {
                color: Color::White,
                ty,
            })
            .or_else(|| {
                self.black_pieces.get_at(bit_pos).map(|ty| Piece {
                    color: Color::Black,
                    ty,
                })
            })
    }

    #[inline]
    pub fn get_pieces(&self, color: Color) -> &Pieces {
        match color {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces,
        }
    }

    pub fn clear(&mut self, bit_pos: u64) {
        if !self.white_pieces.clear(bit_pos) {
            self.black_pieces.clear(bit_pos);
        }
    }

    pub fn set(&mut self, bit_pos: u64, piece: Option<Piece>) {
        self.clear(bit_pos);

        match piece {
            Some(Piece {
                color: Color::White,
                ty,
            }) => {
                self.white_pieces.all |= bit_pos;
                *self.white_pieces.get_mut(ty) |= bit_pos;
            }
            Some(Piece {
                color: Color::Black,
                ty,
            }) => {
                self.black_pieces.all |= bit_pos;
                *self.black_pieces.get_mut(ty) |= bit_pos;
            }
            None => {}
        }
    }

    pub fn check_attack(&self, color: Color) -> u64 {
        let mut attack = 0;

        let (pieces, other_all) = match color {
            Color::White => (self.white_pieces, self.black_pieces.all),
            Color::Black => (self.black_pieces, self.white_pieces.all),
        };

        match color {
            Color::White => {
                attack |= pieces.pawns << 0o11 & !0x101010101010101;
                attack |= pieces.pawns << 7 & !0x8080808080808080;
            }
            Color::Black => {
                attack |= pieces.pawns >> 0o11 & !0x8080808080808080;
                attack |= pieces.pawns >> 7 & !0x101010101010101;
            }
        }

        attack |= ((pieces.king << 1 | pieces.king << 0o11 | pieces.king >> 7)
            & !0x101010101010101
            | (pieces.king >> 1 | pieces.king >> 0o11 | pieces.king << 7) & !0x8080808080808080
            | pieces.king << 0o10
            | pieces.king >> 0o10)
            & !pieces.all;

        {
            let mut move_r = pieces.queens | pieces.rooks;
            let mut move_l = pieces.queens | pieces.rooks;
            let mut move_u = pieces.queens | pieces.rooks;
            let mut move_d = pieces.queens | pieces.rooks;

            let mut move_ru = pieces.queens | pieces.bishops;
            let mut move_lu = pieces.queens | pieces.bishops;
            let mut move_rd = pieces.queens | pieces.bishops;
            let mut move_ld = pieces.queens | pieces.bishops;

            for _ in 1..8 {
                move_r = (move_r & !other_all) << 1 & !0x101010101010101 & !pieces.all;
                move_l = (move_l & !other_all) >> 1 & !0x8080808080808080 & !pieces.all;
                move_u = (move_u & !other_all) << 0o10 & !pieces.all;
                move_d = (move_d & !other_all) >> 0o10 & !pieces.all;

                move_ru = (move_ru & !other_all) << 0o11 & !0x101010101010101 & !pieces.all;
                move_lu = (move_lu & !other_all) << 7 & !0x8080808080808080 & !pieces.all;
                move_rd = (move_rd & !other_all) >> 7 & !0x101010101010101 & !pieces.all;
                move_ld = (move_ld & !other_all) >> 0o11 & !0x8080808080808080 & !pieces.all;

                let move_all =
                    move_r | move_l | move_u | move_d | move_ru | move_lu | move_rd | move_ld;

                attack |= move_all;

                if move_all == 0 {
                    break;
                }
            }
        }

        attack |= ((pieces.knights << 0o21 | pieces.knights >> 0o17) & !0x101010101010101
            | (pieces.knights << 0o17 | pieces.knights >> 0o21) & !0x8080808080808080
            | (pieces.knights << 0o12 | pieces.knights >> 6) & !0x303030303030303
            | (pieces.knights << 6 | pieces.knights >> 0o12) & !0xc0c0c0c0c0c0c0c0)
            & !pieces.all;

        attack
    }

    pub fn is_legal(&self, color: Color, mv: Move) -> bool {
        let pieces_all = self.get_pieces(color).all;
        match mv.ty {
            MoveType::King => {
                let king = self.get_pieces(color).king & 1 << mv.from;

                if ((king << 1 | king << 0o11 | king >> 7) & !0x101010101010101
                    | (king >> 1 | king >> 0o11 | king << 7) & !0x8080808080808080
                    | king << 0o10
                    | king >> 0o10)
                    & !pieces_all
                    & 1 << mv.to
                    == 0
                {
                    return false;
                }
            }
            MoveType::Queen => {
                let other_all = self.get_pieces(color.inv()).all;
                let to_square = 1 << mv.to;

                let queen = self.get_pieces(color).queens & 1 << mv.from;
                if queen == 0 {
                    return false;
                }
                'queen_block: loop {
                    // move r: step = (step & !other_all) << 1 & !0x101010101010101 & !pieces_all;
                    // move l: step = (step & !other_all) >> 1 & !0x8080808080808080 & !pieces_all;
                    // move u: step = (step & !other_all) << 0o10 & !pieces_all;
                    // move d: step = (step & !other_all) >> 0o10 & !pieces_all;

                    // move ru: step = (step & !other_all) << 0o11 & !0x101010101010101 & !pieces_all;
                    // move lu: step = (step & !other_all) << 7 & !0x8080808080808080 & !pieces_all;
                    // move rd: step = (step & !other_all) >> 7 & !0x101010101010101 & !pieces_all;
                    // move ld: step = (step & !other_all) >> 0o11 & !0x8080808080808080 & !pieces_all;

                    let mut step = queen;
                    // right
                    loop {
                        step = (step & !other_all) << 1 & !0x101010101010101 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'queen_block;
                        }
                    }

                    step = queen;
                    // left
                    loop {
                        step = (step & !other_all) >> 1 & !0x8080808080808080 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'queen_block;
                        }
                    }

                    step = queen;
                    // up
                    loop {
                        step = (step & !other_all) << 0o10 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'queen_block;
                        }
                    }

                    step = queen;
                    // down
                    loop {
                        step = (step & !other_all) >> 0o10 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'queen_block;
                        }
                    }

                    step = queen;
                    // right up
                    loop {
                        step = (step & !other_all) << 0o11 & !0x101010101010101 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'queen_block;
                        }
                    }

                    step = queen;
                    // left up
                    loop {
                        step = (step & !other_all) << 7 & !0x8080808080808080 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'queen_block;
                        }
                    }

                    step = queen;
                    // right down
                    loop {
                        step = (step & !other_all) >> 7 & !0x101010101010101 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'queen_block;
                        }
                    }

                    step = queen;
                    // left down
                    loop {
                        step = (step & !other_all) >> 0o11 & !0x8080808080808080 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'queen_block;
                        }
                    }

                    return false;
                }
            }
            MoveType::Rook => {
                let other_all = self.get_pieces(color.inv()).all;
                let to_square = 1 << mv.to;

                let rook = self.get_pieces(color).rooks & 1 << mv.from;
                if rook == 0 {
                    return false;
                }
                'rook_block: loop {
                    let mut step = rook;
                    // right
                    loop {
                        step = (step & !other_all) << 1 & !0x101010101010101 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'rook_block;
                        }
                    }

                    step = rook;
                    // left
                    loop {
                        step = (step & !other_all) >> 1 & !0x8080808080808080 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'rook_block;
                        }
                    }

                    step = rook;
                    // up
                    loop {
                        step = (step & !other_all) << 0o10 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'rook_block;
                        }
                    }

                    step = rook;
                    // down
                    loop {
                        step = (step & !other_all) >> 0o10 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'rook_block;
                        }
                    }

                    return false;
                }
            }
            MoveType::Bishop => {
                let other_all = self.get_pieces(color.inv()).all;
                let to_square = 1 << mv.to;

                let bishop = self.get_pieces(color).bishops & 1 << mv.from;
                if bishop == 0 {
                    return false;
                }
                'bishop_block: loop {
                    let mut step = bishop;
                    // right up
                    loop {
                        step = (step & !other_all) << 0o11 & !0x101010101010101 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'bishop_block;
                        }
                    }

                    step = bishop;
                    // left up
                    loop {
                        step = (step & !other_all) << 7 & !0x8080808080808080 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'bishop_block;
                        }
                    }

                    step = bishop;
                    // right down
                    loop {
                        step = (step & !other_all) >> 7 & !0x101010101010101 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'bishop_block;
                        }
                    }

                    step = bishop;
                    // left down
                    loop {
                        step = (step & !other_all) >> 0o11 & !0x8080808080808080 & !pieces_all;
                        if step == 0 {
                            break;
                        }
                        if step == to_square {
                            break 'bishop_block;
                        }
                    }

                    return false;
                }
            }
            MoveType::Knight => {
                let knight = self.get_pieces(color).knights & 1 << mv.from;

                if ((knight << 0o21 | knight >> 0o17) & !0x101010101010101
                    | (knight << 0o17 | knight >> 0o21) & !0x8080808080808080
                    | (knight << 0o12 | knight >> 6) & !0x303030303030303
                    | (knight << 6 | knight >> 0o12) & !0xc0c0c0c0c0c0c0c0)
                    & !pieces_all
                    & 1 << mv.to
                    == 0
                {
                    return false;
                }
            }
            MoveType::Pawn
            | MoveType::PawnQueenPromotion
            | MoveType::PawnRookPromotion
            | MoveType::PawnBishopPromotion
            | MoveType::PawnKnightPromotion => {
                let all = self.white_pieces.all | self.black_pieces.all;

                match color {
                    Color::White => {
                        let pawn = self.white_pieces.pawns & 1 << mv.from;
                        let other_all = self.black_pieces.all;

                        if (pawn << 0o10 & !all
                            | (pawn << 7 & !0x8080808080808080 | pawn << 0o11 & !0x101010101010101)
                                & other_all)
                            & 1 << mv.to
                            == 0
                        {
                            return false;
                        }

                        if mv.ty != MoveType::Pawn && 1u64 << mv.to & 0xff << 0o70 == 0 {
                            return false;
                        }
                    }
                    Color::Black => {
                        let pawn = self.black_pieces.pawns & 1 << mv.from;
                        let other_all = self.white_pieces.all;

                        if (pawn >> 0o10 & !all
                            | (pawn >> 0o11 & !0x8080808080808080 | pawn >> 7 & !0x101010101010101)
                                & other_all)
                            & 1 << mv.to
                            == 0
                        {
                            return false;
                        }

                        if mv.ty != MoveType::Pawn && 1u64 << mv.to & 0xff == 0 {
                            return false;
                        }
                    }
                }
            }
            MoveType::PawnLeap => {
                let all = self.white_pieces.all | self.black_pieces.all;

                match color {
                    Color::White => {
                        let pawn = self.white_pieces.pawns & 1 << mv.from;

                        if pawn << 0o20 & 1 << mv.to & !all & 0xff00_0000 == 0 {
                            return false;
                        }
                    }
                    Color::Black => {
                        let pawn = self.black_pieces.pawns & 1 << mv.from;

                        if pawn >> 0o20 & 1 << mv.to & !all & 0xff_0000_0000 == 0 {
                            return false;
                        }
                    }
                }
            }
            MoveType::PawnEnPassant => {
                if self.prev_move.ty != MoveType::PawnLeap {
                    return false;
                }
                match color {
                    Color::White => {
                        let pawn = self.white_pieces.pawns & 1 << mv.from;

                        if mv.to != self.prev_move.to + 0o10 {
                            return false;
                        }

                        if (pawn << 7 & !0x8080808080808080 | pawn << 0o11 & !0x101010101010101)
                            & 1 << mv.to
                            == 0
                        {
                            return false;
                        }
                    }
                    Color::Black => {
                        let pawn = self.black_pieces.pawns & 1 << mv.from;

                        if mv.to != self.prev_move.to - 0o10 {
                            return false;
                        }

                        if (pawn >> 0o11 & !0x8080808080808080 | pawn >> 7 & !0x101010101010101)
                            & 1 << mv.to
                            == 0
                        {
                            return false;
                        }
                    }
                }
            }
            MoveType::Castle => {
                let all = self.white_pieces.all | self.black_pieces.all;

                match mv.to {
                    2 => {
                        if color == Color::Black {
                            return false;
                        }
                        if all & 0xe != 0 {
                            return false;
                        }
                        if !self.flags.contains(ChessFlags::WHITE_QUEENS_CASTLE) {
                            return false;
                        }
                        if self.check_attack(Color::Black) & 0x1c != 0 {
                            return false;
                        }
                        return true;
                    }
                    6 => {
                        if color == Color::Black {
                            return false;
                        }
                        if all & 0x60 != 0 {
                            return false;
                        }
                        if !self.flags.contains(ChessFlags::WHITE_KINGS_CASTLE) {
                            return false;
                        }
                        if self.check_attack(Color::Black) & 0x70 != 0 {
                            return false;
                        }
                        return true;
                    }
                    0o72 => {
                        if color == Color::White {
                            return false;
                        }
                        if all & 0xe << 0o70 != 0 {
                            return false;
                        }
                        if !self.flags.contains(ChessFlags::BLACK_QUEENS_CASTLE) {
                            return false;
                        }
                        if self.check_attack(Color::White) & 0x1c << 0o70 != 0 {
                            return false;
                        }
                        return true;
                    }
                    0o76 => {
                        if color == Color::White {
                            return false;
                        }
                        if all & 0x60 << 0o70 != 0 {
                            return false;
                        }
                        if !self.flags.contains(ChessFlags::BLACK_KINGS_CASTLE) {
                            return false;
                        }
                        if self.check_attack(Color::Black) & 0x70 << 0o70 != 0 {
                            return false;
                        }
                        return true;
                    }
                    _ => return false,
                }
            }
        }

        let mut board = *self;
        board.perform_move(mv);
        board.check_attack(color.inv()) & board.get_pieces(color).king == 0
    }

    pub fn moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let (pieces, other_all, other_attack) = match color {
            Color::White => (
                self.white_pieces,
                self.black_pieces.all,
                self.check_attack(Color::Black),
            ),
            Color::Black => (
                self.black_pieces,
                self.white_pieces.all,
                self.check_attack(Color::White),
            ),
        };

        let mut push_move = |mv: Move, use_other_attack: bool| {
            let (attack, king) = if use_other_attack {
                (other_attack, pieces.king)
            } else {
                let mut board = *self;
                board.perform_move(mv);

                (
                    board.check_attack(color.inv()),
                    board.get_pieces(color).king,
                )
            };

            if attack & king == 0 {
                moves.push(mv);
            }
        };

        let all = self.white_pieces.all | self.black_pieces.all;
        match color {
            Color::White => {
                if self.flags.contains(ChessFlags::WHITE_KINGS_CASTLE)
                    && other_attack & 0x70 == 0
                    && all & 0x60 == 0
                {
                    push_move(
                        Move {
                            from: 4,
                            to: 6,
                            ty: MoveType::Castle,
                        },
                        true,
                    );
                }
                if self.flags.contains(ChessFlags::WHITE_QUEENS_CASTLE)
                    && other_attack & 0x1c == 0
                    && all & 0xe == 0
                {
                    push_move(
                        Move {
                            from: 4,
                            to: 2,
                            ty: MoveType::Castle,
                        },
                        true,
                    );
                }

                if self.prev_move.ty == MoveType::PawnLeap {
                    if 1 << (self.prev_move.to + 1) & pieces.pawns & !0x101010101010101 != 0 {
                        push_move(
                            Move {
                                from: self.prev_move.to + 1,
                                to: self.prev_move.to + 0o10,
                                ty: MoveType::PawnEnPassant,
                            },
                            false,
                        );
                    }
                    if 1 << (self.prev_move.to - 1) & pieces.pawns & !0x8080808080808080 != 0 {
                        push_move(
                            Move {
                                from: self.prev_move.to - 1,
                                to: self.prev_move.to + 0o10,
                                ty: MoveType::PawnEnPassant,
                            },
                            false,
                        );
                    }
                }

                let pawn_fwd = pieces.pawns << 0o10 & !all;

                for bit in BitIterator(pawn_fwd) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 - 0o10,
                            to: bit.trailing_zeros() as _,
                            ty: if bit & 0xff << 0o70 == 0 {
                                MoveType::Pawn
                            } else {
                                MoveType::PawnQueenPromotion
                            },
                        },
                        false,
                    );
                }
                for bit in BitIterator(pawn_fwd << 0o10 & !all & 0xff00_0000) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 - 0o20,
                            to: bit.trailing_zeros() as _,
                            ty: MoveType::PawnLeap,
                        },
                        false,
                    );
                }
                for bit in BitIterator(pieces.pawns << 0o11 & !0x101010101010101 & other_all) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 - 0o11,
                            to: bit.trailing_zeros() as _,
                            ty: if bit & 0xff << 0o70 == 0 {
                                MoveType::Pawn
                            } else {
                                MoveType::PawnQueenPromotion
                            },
                        },
                        false,
                    );
                }
                for bit in BitIterator(pieces.pawns << 7 & !0x8080808080808080 & other_all) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 - 7,
                            to: bit.trailing_zeros() as _,
                            ty: if bit & 0xff << 0o70 == 0 {
                                MoveType::Pawn
                            } else {
                                MoveType::PawnQueenPromotion
                            },
                        },
                        false,
                    );
                }
            }
            Color::Black => {
                if self.flags.contains(ChessFlags::BLACK_KINGS_CASTLE)
                    && other_attack & 0x70 << 0o70 == 0
                {
                    push_move(
                        Move {
                            from: 4,
                            to: 6,
                            ty: MoveType::Castle,
                        },
                        true,
                    );
                }
                if self.flags.contains(ChessFlags::BLACK_QUEENS_CASTLE)
                    && other_attack & 0x1c << 0o70 == 0
                {
                    push_move(
                        Move {
                            from: 4,
                            to: 2,
                            ty: MoveType::Castle,
                        },
                        true,
                    );
                }

                if self.prev_move.ty == MoveType::PawnLeap {
                    if 1 << (self.prev_move.to + 1) & pieces.pawns & !0x101010101010101 != 0 {
                        push_move(
                            Move {
                                from: self.prev_move.to + 1,
                                to: self.prev_move.to - 0o10,
                                ty: MoveType::PawnEnPassant,
                            },
                            false,
                        );
                    }
                    if 1 << (self.prev_move.to - 1) & pieces.pawns & !0x8080808080808080 != 0 {
                        push_move(
                            Move {
                                from: self.prev_move.to - 1,
                                to: self.prev_move.to - 0o10,
                                ty: MoveType::PawnEnPassant,
                            },
                            false,
                        );
                    }
                }

                let pawn_fwd = pieces.pawns >> 0o10 & !all;

                for bit in BitIterator(pawn_fwd) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 + 0o10,
                            to: bit.trailing_zeros() as _,
                            ty: if bit & 0xff == 0 {
                                MoveType::Pawn
                            } else {
                                MoveType::PawnQueenPromotion
                            },
                        },
                        false,
                    );
                }
                for bit in BitIterator(pawn_fwd >> 0o10 & !all & 0xff_0000_0000) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 + 0o20,
                            to: bit.trailing_zeros() as _,
                            ty: MoveType::PawnLeap,
                        },
                        false,
                    );
                }
                for bit in BitIterator(pieces.pawns >> 0o11 & !0x8080808080808080 & other_all) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 + 0o11,
                            to: bit.trailing_zeros() as _,
                            ty: if bit & 0xff == 0 {
                                MoveType::Pawn
                            } else {
                                MoveType::PawnQueenPromotion
                            },
                        },
                        false,
                    );
                }
                for bit in BitIterator(pieces.pawns >> 7 & !0x101010101010101 & other_all) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 + 7,
                            to: bit.trailing_zeros() as _,
                            ty: if bit & 0xff == 0 {
                                MoveType::Pawn
                            } else {
                                MoveType::PawnQueenPromotion
                            },
                        },
                        false,
                    );
                }
            }
        }

        {
            let king_moves = ((pieces.king << 1 | pieces.king << 0o11 | pieces.king >> 7)
                & !0x101010101010101
                | (pieces.king >> 1 | pieces.king >> 0o11 | pieces.king << 7)
                    & !0x8080808080808080
                | pieces.king << 0o10
                | pieces.king >> 0o10)
                & !pieces.all;
            for bit in BitIterator(king_moves) {
                push_move(
                    Move {
                        from: pieces.king.trailing_zeros() as u8,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::King,
                    },
                    bit & other_all == 0 && pieces.king & other_attack == 0,
                );
            }
        }

        {
            let mut move_r = pieces.queens | pieces.rooks;
            let mut move_l = pieces.queens | pieces.rooks;
            let mut move_u = pieces.queens | pieces.rooks;
            let mut move_d = pieces.queens | pieces.rooks;

            let mut move_ru = pieces.queens | pieces.bishops;
            let mut move_lu = pieces.queens | pieces.bishops;
            let mut move_rd = pieces.queens | pieces.bishops;
            let mut move_ld = pieces.queens | pieces.bishops;

            for i in 1..8 {
                move_r = (move_r & !other_all) << 1 & !0x101010101010101 & !pieces.all;
                move_l = (move_l & !other_all) >> 1 & !0x8080808080808080 & !pieces.all;
                move_u = (move_u & !other_all) << 0o10 & !pieces.all;
                move_d = (move_d & !other_all) >> 0o10 & !pieces.all;

                move_ru = (move_ru & !other_all) << 0o11 & !0x101010101010101 & !pieces.all;
                move_lu = (move_lu & !other_all) << 7 & !0x8080808080808080 & !pieces.all;
                move_rd = (move_rd & !other_all) >> 7 & !0x101010101010101 & !pieces.all;
                move_ld = (move_ld & !other_all) >> 0o11 & !0x8080808080808080 & !pieces.all;

                if move_r | move_l | move_u | move_d | move_ru | move_lu | move_rd | move_ld == 0 {
                    break;
                }

                for bit in BitIterator(move_r) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 - i,
                            to: bit.trailing_zeros() as _,
                            ty: if bit >> i & pieces.queens != 0 {
                                MoveType::Queen
                            } else {
                                MoveType::Rook
                            },
                        },
                        false,
                    );
                }

                for bit in BitIterator(move_l) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 + i,
                            to: bit.trailing_zeros() as _,
                            ty: if bit << i & pieces.queens != 0 {
                                MoveType::Queen
                            } else {
                                MoveType::Rook
                            },
                        },
                        false,
                    );
                }

                for bit in BitIterator(move_u) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 - 0o10 * i,
                            to: bit.trailing_zeros() as _,
                            ty: if bit >> (0o10 * i) & pieces.queens != 0 {
                                MoveType::Queen
                            } else {
                                MoveType::Rook
                            },
                        },
                        false,
                    );
                }

                for bit in BitIterator(move_d) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 + 0o10 * i,
                            to: bit.trailing_zeros() as _,
                            ty: if bit << (0o10 * i) & pieces.queens != 0 {
                                MoveType::Queen
                            } else {
                                MoveType::Rook
                            },
                        },
                        false,
                    );
                }

                for bit in BitIterator(move_ru) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 - 0o11 * i,
                            to: bit.trailing_zeros() as _,
                            ty: if bit >> (0o11 * i) & pieces.queens != 0 {
                                MoveType::Queen
                            } else {
                                MoveType::Bishop
                            },
                        },
                        false,
                    );
                }

                for bit in BitIterator(move_lu) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 - 7 * i,
                            to: bit.trailing_zeros() as _,
                            ty: if bit >> (7 * i) & pieces.queens != 0 {
                                MoveType::Queen
                            } else {
                                MoveType::Bishop
                            },
                        },
                        false,
                    );
                }

                for bit in BitIterator(move_rd) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 + 7 * i,
                            to: bit.trailing_zeros() as _,
                            ty: if bit << (7 * i) & pieces.queens != 0 {
                                MoveType::Queen
                            } else {
                                MoveType::Bishop
                            },
                        },
                        false,
                    );
                }

                for bit in BitIterator(move_ld) {
                    push_move(
                        Move {
                            from: bit.trailing_zeros() as u8 + 0o11 * i,
                            to: bit.trailing_zeros() as _,
                            ty: if bit << (0o11 * i) & pieces.queens != 0 {
                                MoveType::Queen
                            } else {
                                MoveType::Bishop
                            },
                        },
                        false,
                    );
                }
            }
        }

        {
            // let knight_moves = (0
            //     | (pieces.knights << 0o21 | pieces.knights >> 0o17) & !0x101010101010101
            //     | (pieces.knights << 0o17 | pieces.knights >> 0o21) & !0x8080808080808080
            //     | (pieces.knights << 0o12 | pieces.knights >> 6) & !0x303030303030303
            //     | (pieces.knights << 6 | pieces.knights >> 0o12) & !0xc0c0c0c0c0c0c0c0
            //     | 0)
            //     & !pieces.all;
            for bit in BitIterator(pieces.knights << 0o21 & !0x101010101010101 & !pieces.all) {
                push_move(
                    Move {
                        from: bit.trailing_zeros() as u8 - 0o21,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::Knight,
                    },
                    false,
                );
            }
            for bit in BitIterator(pieces.knights >> 0o17 & !0x101010101010101 & !pieces.all) {
                push_move(
                    Move {
                        from: bit.trailing_zeros() as u8 + 0o17,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::Knight,
                    },
                    false,
                );
            }

            for bit in BitIterator(pieces.knights << 0o17 & !0x8080808080808080 & !pieces.all) {
                push_move(
                    Move {
                        from: bit.trailing_zeros() as u8 - 0o17,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::Knight,
                    },
                    false,
                );
            }
            for bit in BitIterator(pieces.knights >> 0o21 & !0x8080808080808080 & !pieces.all) {
                push_move(
                    Move {
                        from: bit.trailing_zeros() as u8 + 0o21,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::Knight,
                    },
                    false,
                );
            }

            for bit in BitIterator(pieces.knights << 0o12 & !0x303030303030303 & !pieces.all) {
                push_move(
                    Move {
                        from: bit.trailing_zeros() as u8 - 0o12,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::Knight,
                    },
                    false,
                );
            }
            for bit in BitIterator(pieces.knights >> 6 & !0x303030303030303 & !pieces.all) {
                push_move(
                    Move {
                        from: bit.trailing_zeros() as u8 + 6,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::Knight,
                    },
                    false,
                );
            }

            for bit in BitIterator(pieces.knights << 6 & !0xc0c0c0c0c0c0c0c0 & !pieces.all) {
                push_move(
                    Move {
                        from: bit.trailing_zeros() as u8 - 6,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::Knight,
                    },
                    false,
                );
            }
            for bit in BitIterator(pieces.knights >> 0o12 & !0xc0c0c0c0c0c0c0c0 & !pieces.all) {
                push_move(
                    Move {
                        from: bit.trailing_zeros() as u8 + 0o12,
                        to: bit.trailing_zeros() as _,
                        ty: MoveType::Knight,
                    },
                    false,
                );
            }
        }

        moves
    }

    pub fn perform_move(&mut self, mv: Move) {
        self.prev_move = mv;

        let color = if 1 << mv.from & self.white_pieces.all != 0 {
            self.white_pieces.all &= !(1 << mv.from);
            self.white_pieces.all |= 1 << mv.to;

            Color::White
        } else {
            self.black_pieces.all &= !(1 << mv.from);
            self.black_pieces.all |= 1 << mv.to;

            Color::Black
        };
        match mv.ty {
            MoveType::King => match color {
                Color::White => {
                    self.white_pieces.king = 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.king = 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::Queen => match color {
                Color::White => {
                    self.white_pieces.queens &= !(1 << mv.from);
                    self.white_pieces.queens |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.queens &= !(1 << mv.from);
                    self.black_pieces.queens |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::Rook => match color {
                Color::White => {
                    self.white_pieces.rooks &= !(1 << mv.from);
                    self.white_pieces.rooks |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.rooks &= !(1 << mv.from);
                    self.black_pieces.rooks |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::Bishop => match color {
                Color::White => {
                    self.white_pieces.bishops &= !(1 << mv.from);
                    self.white_pieces.bishops |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.bishops &= !(1 << mv.from);
                    self.black_pieces.bishops |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::Knight => match color {
                Color::White => {
                    self.white_pieces.knights &= !(1 << mv.from);
                    self.white_pieces.knights |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.knights &= !(1 << mv.from);
                    self.black_pieces.knights |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::Pawn => match color {
                Color::White => {
                    self.white_pieces.pawns &= !(1 << mv.from);
                    self.white_pieces.pawns |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.pawns &= !(1 << mv.from);
                    self.black_pieces.pawns |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::PawnLeap => match color {
                Color::White => {
                    self.white_pieces.pawns &= !(1 << mv.from);
                    self.white_pieces.pawns |= 1 << mv.to;
                }
                Color::Black => {
                    self.black_pieces.pawns &= !(1 << mv.from);
                    self.black_pieces.pawns |= 1 << mv.to;
                }
            },
            MoveType::PawnEnPassant => match color {
                Color::White => {
                    self.black_pieces.all &= !(1 << (mv.to - 0o10));
                    self.black_pieces.pawns &= !(1 << (mv.to - 0o10));

                    self.white_pieces.pawns &= !(1 << mv.from);
                    self.white_pieces.pawns |= 1 << mv.to;
                }
                Color::Black => {
                    self.white_pieces.all &= !(1 << (mv.to + 0o10));
                    self.white_pieces.pawns &= !(1 << (mv.to + 0o10));

                    self.black_pieces.pawns &= !(1 << mv.from);
                    self.black_pieces.pawns |= 1 << mv.to;
                }
            },
            MoveType::PawnQueenPromotion => match color {
                Color::White => {
                    self.white_pieces.pawns &= !(1 << mv.from);
                    self.white_pieces.queens |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.pawns &= !(1 << mv.from);
                    self.black_pieces.queens |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::PawnRookPromotion => match color {
                Color::White => {
                    self.white_pieces.pawns &= !(1 << mv.from);
                    self.white_pieces.rooks |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.pawns &= !(1 << mv.from);
                    self.black_pieces.rooks |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::PawnBishopPromotion => match color {
                Color::White => {
                    self.white_pieces.pawns &= !(1 << mv.from);
                    self.white_pieces.bishops |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.pawns &= !(1 << mv.from);
                    self.black_pieces.bishops |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::PawnKnightPromotion => match color {
                Color::White => {
                    self.white_pieces.pawns &= !(1 << mv.from);
                    self.white_pieces.knights |= 1 << mv.to;

                    self.black_pieces.clear(1 << mv.to);
                }
                Color::Black => {
                    self.black_pieces.pawns &= !(1 << mv.from);
                    self.black_pieces.knights |= 1 << mv.to;

                    self.white_pieces.clear(1 << mv.to);
                }
            },
            MoveType::Castle => match mv.to {
                2 => {
                    self.white_pieces.king = 4;

                    self.white_pieces.all &= !1;
                    self.white_pieces.rooks &= !1;

                    self.white_pieces.all |= 8;
                    self.white_pieces.rooks |= 8;
                }
                6 => {
                    self.white_pieces.king = 0x40;

                    self.white_pieces.all &= !0x80;
                    self.white_pieces.rooks &= !0x80;

                    self.white_pieces.all |= 0x20;
                    self.white_pieces.rooks |= 0x20;
                }
                0o72 => {
                    self.black_pieces.king = 1 << 0o72;

                    self.black_pieces.all &= !(1 << 0o70);
                    self.black_pieces.rooks &= !(1 << 0o70);

                    self.black_pieces.all |= 1 << 0o73;
                    self.black_pieces.rooks |= 1 << 0o73;
                }
                0o76 => {
                    self.black_pieces.king = 1 << 0o76;

                    self.black_pieces.all &= !(1 << 0o77);
                    self.black_pieces.rooks &= !(1 << 0o77);

                    self.black_pieces.all |= 1 << 0o75;
                    self.black_pieces.rooks |= 1 << 0o75;
                }
                _ => panic!("Illigal castle accidentally cought"),
            },
        }

        self.flags.remove(if self.white_pieces.king == 0x10 {
            ChessFlags::empty()
        } else {
            ChessFlags::WHITE_KINGS_CASTLE | ChessFlags::WHITE_QUEENS_CASTLE
        });
        self.flags
            .remove(if self.black_pieces.king == 0x10 << 0o70 {
                ChessFlags::empty()
            } else {
                ChessFlags::WHITE_KINGS_CASTLE | ChessFlags::WHITE_QUEENS_CASTLE
            });
        self.flags.remove(if self.white_pieces.rooks & 1 != 0 {
            ChessFlags::empty()
        } else {
            ChessFlags::WHITE_QUEENS_CASTLE
        });
        self.flags.remove(if self.white_pieces.rooks & 0x80 != 0 {
            ChessFlags::empty()
        } else {
            ChessFlags::WHITE_KINGS_CASTLE
        });
        self.flags
            .remove(if self.white_pieces.rooks & 1 << 0o70 != 0 {
                ChessFlags::empty()
            } else {
                ChessFlags::BLACK_QUEENS_CASTLE
            });
        self.flags
            .remove(if self.white_pieces.rooks & 1 << 0o77 != 0 {
                ChessFlags::empty()
            } else {
                ChessFlags::BLACK_KINGS_CASTLE
            });
    }

    pub fn print(&self, color: Color) {
        match color {
            Color::White => {
                for i in (0..64).step_by(8).rev() {
                    print!("{}", 1 + i / 8);
                    for j in i..i + 8 {
                        print!(
                            " {}",
                            match self.get_at(1 << j) {
                                None => {
                                    if (j ^ j >> 3) & 1 == 0 {
                                        '\u{25FC}'
                                    } else {
                                        '\u{25FB}'
                                    }
                                }
                                Some(piece) => piece.to_char(),
                            }
                        );
                    }
                    println!();
                }
            }
            Color::Black => {
                for i in (0..64).step_by(8) {
                    print!("{}", 1 + i / 8);
                    for j in i..i + 8 {
                        print!(
                            " {}",
                            match self.get_at(1 << j) {
                                None => {
                                    if (j ^ j >> 3) & 1 == 0 {
                                        '\u{25FC}'
                                    } else {
                                        '\u{25FB}'
                                    }
                                }
                                Some(piece) => piece.to_char(),
                            }
                        );
                    }
                    println!();
                }
            }
        }
        print!(" ");
        for ch in 'a'..='h' {
            print!(" {}", ch);
        }
        println!();
        println!();
    }

    pub fn print_moves(&self, color: Color) {
        match color {
            Color::White => println!("White's moves:"),
            Color::Black => println!("Black's moves:"),
        }
        for mv in self.moves(color) {
            println!(
                "  {} : {}->{}  // move.type={:?}",
                self.get_at(1 << mv.from)
                    .map(|p| p.to_char())
                    .unwrap_or('#'),
                to_chess_pos(mv.from),
                to_chess_pos(mv.to),
                mv.ty,
            );
        }
        println!();
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Piece {
    pub fn to_char(&self) -> char {
        match self {
            Piece {
                color: Color::White,
                ty: PieceType::King,
            } => '\u{2654}',
            Piece {
                color: Color::White,
                ty: PieceType::Queen,
            } => '\u{2655}',
            Piece {
                color: Color::White,
                ty: PieceType::Rook,
            } => '\u{2656}',
            Piece {
                color: Color::White,
                ty: PieceType::Bishop,
            } => '\u{2657}',
            Piece {
                color: Color::White,
                ty: PieceType::Knight,
            } => '\u{2658}',
            Piece {
                color: Color::White,
                ty: PieceType::Pawn,
            } => '\u{2659}',
            Piece {
                color: Color::Black,
                ty: PieceType::King,
            } => '\u{265A}',
            Piece {
                color: Color::Black,
                ty: PieceType::Queen,
            } => '\u{265B}',
            Piece {
                color: Color::Black,
                ty: PieceType::Rook,
            } => '\u{265C}',
            Piece {
                color: Color::Black,
                ty: PieceType::Bishop,
            } => '\u{265D}',
            Piece {
                color: Color::Black,
                ty: PieceType::Knight,
            } => '\u{265E}',
            Piece {
                color: Color::Black,
                ty: PieceType::Pawn,
            } => '\u{265F}',
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct BitIterator(pub u64);

impl Iterator for BitIterator {
    type Item = u64;

    #[inline]
    fn next(&mut self) -> Option<u64> {
        if self.0 == 0 {
            None
        } else {
            let prev = self.0;
            self.0 &= self.0 - 1;

            Some(prev ^ self.0)
        }
    }
}

// #[test]
// fn bit_iterator() {
//     let x = 0b10000001000100101101011;
//
//     let x_str = format!("{:b}", x);
//     println!("{}", x_str);
//     println!("{:->1$}", "", x_str.len());
//     for bit in BitIterator(x) {
//         println!("{:>1$b}", bit, x_str.len());
//     }
// }

fn chess_pos(chs: &[u8]) -> u8 {
    assert_eq!(chs.len(), 2);
    assert!((b'a'..=b'h').contains(&chs[0]));
    assert!((b'1'..=b'8').contains(&chs[1]));
    8 * (chs[1] - b'1') + (chs[0] - b'a')
}

fn to_chess_pos(x: u8) -> String {
    String::from_utf8([b'a' + (x & 7), b'1' + x / 8].to_vec()).unwrap()
}

fn main() {
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
        board.print_moves(color);
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
    board.print_moves(color);
    println!("attack: 0x{:x}", board.check_attack(color));
    println!();
}
