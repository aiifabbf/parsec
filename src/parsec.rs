pub trait Parser<T>: Sized {
    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)>;

    // fn and_then<T2, P2>(self, another: P2) -> AndThen<Self, P2>
    // where
    //     P2: Parser<T2>,
    // {
    //     AndThen(self, another)
    // }
}

struct Any;

impl Parser<char> for Any {
    fn parse<'a>(&self, input: &'a str) -> Option<(char, &'a str)> {
        if let Some(first) = input.chars().next() {
            Some((first.clone(), &input[first.len_utf8()..]))
        } else {
            None
        }
    }
}

pub fn any() -> impl Parser<char> {
    Any // 其实这是个unit type，根本不占空间，我怀疑这里会自动优化
        // Satisfy(|c: &char| true) // 也可以用Satisfy构造，但是语义上要占用一个closure的空间
}

struct Eof;

impl Parser<()> for Eof {
    fn parse<'a>(&self, input: &'a str) -> Option<((), &'a str)> {
        if input.is_empty() {
            Some(((), input))
        } else {
            None
        }
    }
}

pub fn eof() -> impl Parser<()> {
    Eof
}

struct Satisfy<F>(F);

impl<F> Parser<char> for Satisfy<F>
where
    F: Fn(&char) -> bool, // Fn(char) -> bool
{
    fn parse<'a>(&self, input: &'a str) -> Option<(char, &'a str)> {
        if let Some(first) = input.chars().next() {
            if (self.0)(&first) {
                Some((first.clone(), &input[first.len_utf8()..]))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn satisfy<F>(f: F) -> impl Parser<char>
where
    F: Fn(&char) -> bool,
{
    Satisfy(f)
}

struct Char(char);

impl Parser<char> for Char {
    fn parse<'a>(&self, input: &'a str) -> Option<(char, &'a str)> {
        if let Some(first) = input.chars().next() {
            if first == self.0 {
                Some((first.clone(), &input[first.len_utf8()..]))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn char(c: char) -> impl Parser<char> {
    Char(c)
    // Satisfy(move |v: &char| c == *v)
}

pub fn space() -> impl Parser<char> {
    char(' ')
}

pub fn newline() -> impl Parser<char> {
    char('\n')
}

pub fn tab() -> impl Parser<char> {
    char('\t')
}

pub fn one_of<'a>(array: &'a [char]) -> impl Parser<char> + 'a {
    satisfy(move |c| array.contains(c)) // 一定要move，array需要移动到closure里面
}

struct Str<'a>(&'a str);

impl<'b> Parser<&'b str> for Str<'b> {
    fn parse<'a>(&self, input: &'a str) -> Option<(&'b str, &'a str)> {
        if input
            .chars()
            .zip(self.0.chars())
            .filter(|(v, w)| v == w)
            .count()
            == self.0.len()
        {
            Some((self.0, &input[self.0.len()..]))
        } else {
            None
        }
    }
}

pub fn string(pattern: &str) -> impl Parser<&str> {
    Str(pattern)
}

struct Many<P>(P);

// impl<P, T> Parser<&[T]> for Many<P> where P: Parser<T> {
//     fn parse<'a>(&self, input: &'a str) -> Option<(&[T], &'a str)> {

//     }
// }

impl<P> Parser<String> for Many<P>
// 不知道怎么改成Parser<&str>呜呜呜
where
    P: Parser<char>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(String, &'a str)> {
        let mut input = input;
        let mut target = String::new();

        loop {
            if let Some((c, remaining)) = self.0.parse(input) {
                input = remaining;
                target.push(c);
            } else {
                break Some((target, input));
            }
        }
    }
}

pub fn many<'a>(parser: impl Parser<char>) -> impl Parser<String> {
    Many(parser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_parse_non_empty() {
        let input = "abc";
        let parser = Any;
        assert_eq!(dbg!(parser.parse(input)), Some(('a', "bc")));
    }

    #[test]
    fn any_parse_empty() {
        let input = "";
        let parser = Any;
        assert_eq!(dbg!(parser.parse(input)), None);
    }

    #[test]
    fn eof_parse_empty() {
        let input = "";
        let parser = Eof;
        assert_eq!(dbg!(parser.parse(input)), Some(((), "")));
    }

    #[test]
    fn eof_parse_non_empty() {
        let input = "a";
        let parser = Eof;
        assert_eq!(dbg!(parser.parse(input)), None);
    }

    #[test]
    fn satisfy_parse_fail() {
        let input = "abc";
        let parser = Satisfy(|c: &char| c.is_ascii_uppercase()); // 这里需要给参数写type hint，有点恶心
        assert_eq!(dbg!(parser.parse(input)), None);
    }

    #[test]
    fn satisfy_parse_succeed() {
        let input = "Abc";
        let parser = satisfy(|c| c.is_ascii_uppercase()); // 搞一个函数就不用了……
        assert_eq!(dbg!(parser.parse(input)), Some(('A', "bc")));
    }

    #[test]
    fn satisfy_parse_digit() {
        let input = "1bc";
        let parser = Satisfy(|c: &char| c.is_digit(10));
        assert_eq!(dbg!(parser.parse(input)), Some(('1', "bc")));
    }

    #[test]
    fn satisfy_parse_non_digit() {
        let input = "abc";
        let parser = Satisfy(|c: &char| c.is_digit(10));
        assert_eq!(dbg!(parser.parse(input)), None);
    }

    #[test]
    fn char_parse_char() {
        let input = "abc";
        let parser = char('a'); // char竟然不是保留字
        let _ = 1 as char; // 竟然可以区分同名函数和type
        assert_eq!(dbg!(parser.parse(input)), Some(('a', "bc")));
    }

    #[test]
    fn char_parse_fail() {
        let input = "abc";
        let parser = char('b');
        assert_eq!(dbg!(parser.parse(input)), None);
    }

    #[test]
    fn one_of_parse_succeed() {
        let input = "c";
        let parser = one_of(&['a', 'b', 'c']);
        assert_eq!(dbg!(parser.parse(input)), Some(('c', "")));
    }

    #[test]
    fn one_of_parse_fail() {
        let input = "c";
        let parser = one_of(&['x', 'y', 'z']);
        assert_eq!(dbg!(parser.parse(input)), None);
    }

    #[test]
    fn string_parse_succeed() {
        let input = "prefixaaaa";
        let parser = string("prefix");
        assert_eq!(dbg!(parser.parse(input)), Some(("prefix", "aaaa")));
    }

    #[test]
    fn many_parse_number() {
        let input = "1234";
        let digit = satisfy(|c| c.is_digit(10));
        let parser = many(digit);
        assert_eq!(dbg!(parser.parse(input)), Some(("1234".to_owned(), "")));
    }
}
