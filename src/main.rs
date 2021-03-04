// struct AndThen<P1, P2>(P1, P2);

// impl<T1, T2, P1, P2> Parser<(T1, T2)> for AndThen<P1, P2>
// where
//     P1: Parser<T1>,
//     P2: Parser<T2>,
// {
//     fn parse<'a>(&self, input: &'a str) -> Option<((T1, T2), &'a str)> {
//         if let Some((a, remaining)) = self.0.parse(input) {
//             if let Some((b, remaining)) = self.1.parse(remaining) {
//                 Some(((a, b), remaining))
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }
mod parsec;

fn main() {}
