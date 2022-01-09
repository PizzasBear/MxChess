
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

