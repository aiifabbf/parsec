use std::fmt::Debug;
use std::marker::PhantomData;
// use std::ops::BitOr; // 本来想实现p1 | p2这种的，无奈又遇到了unconstrained type parameter问题，暂时放一放
use std::str::FromStr;

pub trait Parser<T> {
    // 去掉了: Sized约束。如果不去掉，会使得任何实现了Parser<T>的struct无法变成trait object。
    // 那么联想到Iterator是怎么实现的呢？Iterator有的方法是取self（比如map、zip这一类）、有的方法取&mut self（比如next）。
    // 方法就是不要在trait层面就约束Sized，而是到方法层面约束。在方法后面加where Self: Sized。
    // 虽然我还是不理解为什么Sized就不能变成dyn Trait……

    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)>;

    // fn and_then<T2, P2>(self, another: P2) -> AndThen<Self, P2>
    // where
    //     P2: Parser<T2>,
    // {
    //     AndThen(self, another)
    // }

    /// p*
    fn many(self) -> Many<Self>
    where
        Self: Sized, // 第一次知道还有这种写法。从std的Iterator学过来的
    {
        Many(self)
    } // 我以为many: Parser<char> -> Parser<String>是不可能的……

    /// p+
    fn many1(self) -> Many1<Self>
    where
        Self: Sized,
    {
        Many1(self)
    }

    /// like p1 <|> p2, but backtrack on default
    ///
    /// try to match p1, if success, return what p1 matches; otherwise try to match p2, if success, return what p2 matches.
    fn choice<P>(self, another: P) -> Choice<Self, P>
    where
        Self: Sized,
    {
        // Haskell parsec里的<|>似乎默认是不回溯的
        Choice(self, another)
    }

    /// Parser<T1> -> (T1 -> T2) -> Parser<T2>
    fn map<T2, F>(self, f: F) -> Map<T, Self, F>
    where
        Self: Sized,
        F: Fn(T) -> T2,
    {
        Map(self, f, PhantomData)
    }

    /// Parser<T1> -> (T1 -> Parser<T2>) -> Parser<T2>
    // 其实我到现在还不明白这个and_then可以用在哪里……
    fn and_then<F, T2, P2>(self, f: F) -> AndThen<T, Self, F>
    where
        Self: Sized,
        F: Fn(T) -> P2,
        P2: Parser<T2>,
    {
        AndThen(self, f, PhantomData)
    }

    /// p{n}, match p for n times
    fn count(self, n: usize) -> Count<Self>
    where
        Self: Sized,
    {
        Count(self, n)
    }

    /// p1 <* p2, match p1 then match p2, return what p1 matches
    fn left<T2, P2>(self, another: P2) -> Left<T, Self, T2, P2>
    where
        Self: Sized,
        P2: Parser<T2>,
    {
        Left(self, another, PhantomData)
    }

    /// p1 *> p2, match p1 then match p2, return what p2 matches
    fn right<T2, P2>(self, another: P2) -> Right<T, Self, T2, P2>
    where
        Self: Sized,
    {
        Right(self, another, PhantomData)
    }

    /// p1 *> p <* p2, match p1 then p then p2, return what p matches
    fn between<T1, P1, T2, P2>(self, p1: P1, p2: P2) -> Left<T, Right<T1, P1, T, Self>, T2, P2>
    where
        Self: Sized,
        P1: Parser<T1>,
        P2: Parser<T2>,
    {
        p1.right(self).left(p2)
    }

    /// match p, consume no input even if success
    fn look_ahead(self) -> LookAhead<Self>
    where
        Self: Sized,
    {
        LookAhead(self)
    }

    /// match p and then 0 or more spaces, return what p matches
    fn lexeme(self) -> Left<T, Self, (), Whitespaces>
    where
        Self: Sized,
    {
        self.left(Whitespaces)
    }
    // 和Haskell parsec的不一样，没考虑注释啥的，单纯就是空格

    /// match 0 or more p, separated by separator
    fn separated_by<T2, P2>(self, separator: P2) -> SeparatedBy<T, Self, T2, P2>
    where
        Self: Sized,
        P2: Parser<T2>,
    {
        SeparatedBy(self, separator, PhantomData)
    }

    /// try to match p, if success, consumes input and return (); otherwise returns () and does not consume input
    fn optional(self) -> Optional<T, Self>
    where
        Self: Sized,
    {
        Optional(self, PhantomData)
    }

    /// match 0 or more p, separated by separator and optionally ended by separator
    fn separated_end_by<T2, P2>(self, separator: P2) -> SeparatedEndBy<T, Self, T2, P2>
    where
        Self: Sized,
        P2: Parser<T2>,
    {
        SeparatedEndBy(self, separator, PhantomData)
    }

    // 这两个不知道有什么用……直接写也足够简单了
    fn end_by<T2, P2>(self, separator: P2) -> Many<Left<T, Self, T2, P2>>
    where
        Self: Sized,
        P2: Parser<T2>,
    {
        self.left(separator).many()
    }

    fn end_by1<T2, P2>(self, separator: P2) -> Many1<Left<T, Self, T2, P2>>
    where
        Self: Sized,
        P2: Parser<T2>,
    {
        self.left(separator).many1()
    }

    fn chain_left1<T2, P2>(self, operator: P2) -> ChainLeft1<T, Self, T2, P2>
    where
        Self: Sized,
        P2: Parser<T2>,
    {
        ChainLeft1(self, operator, PhantomData)
    }

    fn chain_right1<T2, P2>(self, operator: P2) -> ChainRight1<T, Self, T2, P2>
    where
        Self: Sized,
        P2: Parser<T2>,
    {
        ChainRight1(self, operator, PhantomData)
    }
}

#[derive(Clone)]
pub struct Any;

impl Parser<char> for Any {
    fn parse<'a>(&self, input: &'a str) -> Option<(char, &'a str)> {
        if let Some(first) = input.chars().next() {
            Some((first.clone(), &input[first.len_utf8()..]))
        } else {
            None
        }
    }
}

/// any 1 character
pub fn any(s: &str) -> Option<(char, &str)> {
    Any.parse(s)
}

// 天哪，我才发现函数签名里面返回值类型写impl Trait和写具体的P类型竟然是不同的！写impl Trait的话，会导致f()只能用Trait里的方法，无法发现P的方法。
// https://stackoverflow.com/questions/64693182/rust-expected-type-found-opaque-type
// 可是如果返回值类型真的复杂到没法写出来呢？或者有些情况根本就写不出类型（比如closure）？别急，你可以写impl Trait + Clone。下面一大堆用satisfy实现的函数就是这样做的。

#[derive(Clone)]
pub struct Eof;

impl Parser<()> for Eof {
    fn parse<'a>(&self, input: &'a str) -> Option<((), &'a str)> {
        if input.is_empty() {
            Some(((), input))
        } else {
            None
        }
    }
}

/// only succeed when input is empty
pub fn eof(s: &str) -> Option<((), &str)> {
    Eof.parse(s)
}

#[derive(Clone)]
pub struct Epsilon;

impl Parser<()> for Epsilon {
    fn parse<'a>(&self, input: &'a str) -> Option<((), &'a str)> {
        Some(((), input))
    }
}

// 不知道这个定义对不对
/// always succeed, consume nothing
pub fn epsilon(s: &str) -> Option<((), &str)> {
    Epsilon.parse(s)
}

// #[derive(Clone)]
// pub struct Always<T>(T);

// impl<T> Parser<T> for Always<T>
// where
//     T: Clone,
// {
//     fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
//         Some((self.0.clone(), input))
//     }
// }

// pub fn always<T>(t: T) -> impl Parser<T>
// where
//     T: 'static + Clone,
// {
//     epsilon().map(move |_| t.clone())
// }

#[derive(Clone)]
pub struct Satisfy<F>(F);
// 不用担心如果F不满足Clone怎么办，根据文档，derive(Clone)其实相当于impl<F> Clone for Satisfy<F> where F: Clone，当且仅当F也满足Clone时才会让Satisfy<F>也满足Clone，非常贴心

impl<F> Parser<char> for Satisfy<F>
where
    F: Fn(char) -> bool, // Fn(char) -> bool
{
    fn parse<'a>(&self, input: &'a str) -> Option<(char, &'a str)> {
        if let Some(first) = input.chars().next() {
            if (self.0)(first) {
                Some((first.clone(), &input[first.len_utf8()..]))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// 1 character c that makes f(c) true
pub fn satisfy<F>(f: F) -> Satisfy<F>
where
    F: Fn(char) -> bool,
{
    Satisfy(f)
}

#[derive(Clone)]
pub struct Char(char);

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

/// 1 particular character c
pub fn char(c: char) -> Char {
    Char(c)
    // Satisfy(move |v: &char| c == *v)
}

/// 1 whitespace character
pub fn whitespace(s: &str) -> Option<(char, &str)> {
    satisfy(|c| c.is_whitespace()).parse(s)
}

#[derive(Clone)]
pub struct Whitespaces;

impl Parser<()> for Whitespaces {
    fn parse<'a>(&self, input: &'a str) -> Option<((), &'a str)> {
        Some(((), input.trim_start()))
    }
}

/// 0 or more whitespace characters
pub fn whitespaces(s: &str) -> Option<((), &str)> {
    Whitespaces.parse(s)
}

/// 1 or more whitespace characters
pub fn gap(s: &str) -> Option<((), &str)> {
    whitespace.many1().map(|_: String| ()).parse(s)
}

/// 1 \n
pub fn newline(s: &str) -> Option<(char, &str)> {
    char('\n').parse(s)
}

/// 1 \t
pub fn tab(s: &str) -> Option<(char, &str)> {
    char('\t').parse(s)
}

/// 1 uppercase character
pub fn upper(s: &str) -> Option<(char, &str)> {
    satisfy(|c| c.is_uppercase()).parse(s)
}

/// 1 lowercase character
pub fn lower(s: &str) -> Option<(char, &str)> {
    satisfy(|c| c.is_lowercase()).parse(s)
}

/// 1 alphanumeric character
pub fn alphanumeric(s: &str) -> Option<(char, &str)> {
    satisfy(|c| c.is_alphanumeric()).parse(s)
}

/// '0'..='9'
pub fn digit(s: &str) -> Option<(char, &str)> {
    satisfy(|c| c.is_digit(10)).parse(s)
}

/// '0'..='9', 'a'..='f' and 'A'..='F'
pub fn hex_digit(s: &str) -> Option<(char, &str)> {
    satisfy(|c| c.is_digit(16)).parse(s)
}

/// 1 character that is an element of the char slice
pub fn one_of<'a>(array: &'a [char]) -> impl Parser<char> + Clone + 'a {
    satisfy(move |c| array.contains(&c)) // 一定要move，array需要移动到closure里面
}

/// 1 character that is not an element of the char slice
pub fn none_of<'a>(array: &'a [char]) -> impl Parser<char> + Clone + 'a {
    satisfy(move |c| !array.contains(&c))
}

#[derive(Clone)]
pub struct Str<'a>(&'a str);
// 为什么这里不用pub呢？

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

/// match a particular string
pub fn string<'a>(pattern: &'a str) -> Str<'a> {
    Str(pattern)
}

#[derive(Clone)]
pub struct Many<P>(P);

// impl<P, T> Parser<&[T]> for Many<P> where P: Parser<T> {
//     fn parse<'a>(&self, input: &'a str) -> Option<(&[T], &'a str)> {

//     }
// }
// 这应该是做不到的

impl<T, P> Parser<Vec<T>> for Many<P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(Vec<T>, &'a str)> {
        let mut input = input;
        let mut target = vec![];

        loop {
            if let Some((a, remaining)) = self.0.parse(input) {
                input = remaining;
                target.push(a);
            } else {
                break Some((target, input));
            }
        }
    }
}

impl<P> Parser<String> for Many<P>
// 不知道怎么改成Parser<&str>呜呜呜
// 我错了，应该是做不到的
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

// 上面为一般的情况实现了many: Parser<T> -> Parser<Vec<T>>、也为char特别实现了many: Parser<char> -> Parser<String>
// 这就带来一个问题，假设p: Parser<char>，那么p.many().parse的类型应该是Parser<Vec<char>>还是Parser<String>呢？
// 所以有时候会出现需要type annotation的情况。

// impl<P> Parser<String> for P
// where
//     P: Parser<Vec<char>>,
// {
//     fn parse<'a>(&self, input: &'a str) -> Option<(String, &'a str)> {
//         if let Some((s, remaining)) = self.parse(input) {
//             Some((s.into_iter().collect(), remaining))
//         } else {
//             None
//         }
//     }
// }
// 妄图通过给所有Parser<Vec<char>>实现Parser<String>来解决p.many()究竟应该是Parser<String>还是Parser<Vec<char>>的问题

#[derive(Clone)]
pub struct Many1<P>(P);

impl<T, P> Parser<Vec<T>> for Many1<P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(Vec<T>, &'a str)> {
        let mut input = input;
        let mut target = vec![];

        loop {
            if let Some((a, remaining)) = self.0.parse(input) {
                input = remaining;
                target.push(a);
            } else {
                break if target.is_empty() {
                    None
                } else {
                    Some((target, input))
                };
            }
        }
    }
}

impl<P> Parser<String> for Many1<P>
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
                break if target.is_empty() {
                    None
                } else {
                    Some((target, input))
                };
            }
        }
    }
}
// 所有都要写两遍，代码还都差不多，好烦哦

#[derive(Clone)]
pub struct Choice<P1, P2>(P1, P2);

impl<T, P1, P2> Parser<T> for Choice<P1, P2>
where
    P1: Parser<T>,
    P2: Parser<T>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
        if let Some((a, remaining)) = self.0.parse(input) {
            Some((a, remaining))
        } else if let Some((b, remaining)) = self.1.parse(input) {
            Some((b, remaining))
        } else {
            None
        }
    }
}

// 这样竟然是不行的
// impl<T, P1, P2> BitOr<P2> for P1
// where
//     P1: Parser<T>,
//     P2: Parser<T>,
// {
//     type Output = Choice<P1, P2>;

//     fn bitor(self, rhs: P2) -> Self::Output {
//         self.choice(rhs)
//     }
// }
// error[E0207]: the type parameter `T` is not constrained by the impl trait, self type, or predicates

// 本来就是简简单单的写法
// pub struct Map<P, F>(P, F);

// 非要搞成这样，就为了不出现没用过的泛型参数
#[derive(Clone)]
pub struct Map<T, P, F>(P, F, PhantomData<T>);

// 讨论
// https://www.reddit.com/r/rust/comments/fkulrf/quick_question_about_unused_generic_type_parameter/
// https://stackoverflow.com/questions/28123445/is-there-any-way-to-work-around-an-unused-type-parameter
// https://github.com/rust-lang/rust/issues/23246

impl<T1, P1, T2, F> Parser<T2> for Map<T1, P1, F>
where
    P1: Parser<T1>,
    F: Fn(T1) -> T2,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T2, &'a str)> {
        if let Some((res, remaining)) = self.0.parse(input) {
            Some(((self.1)(res), remaining))
        } else {
            None
        }
    }
}

// impl<T1, T2, P1, P2> From<P1> for P2
// where
//     P1: Parser<T1>,
//     P2: Parser<T2>,
//     T2: From<T1>
// {
//     fn from(parser: P1) -> P2 {
//         parser.map(|v: T1| v.into())
//     }
// }
// 有map了，应该也不用到这个了

#[derive(Clone)]
pub struct AndThen<T, P, F>(P, F, PhantomData<T>);

impl<T1, P1, T2, P2, F> Parser<T2> for AndThen<T1, P1, F>
where
    P1: Parser<T1>,
    P2: Parser<T2>,
    F: Fn(T1) -> P2,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T2, &'a str)> {
        if let Some((res, remaining)) = self.0.parse(input) {
            (self.1)(res).parse(remaining)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Count<P>(P, usize);

impl<T, P> Parser<Vec<T>> for Count<P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(Vec<T>, &'a str)> {
        let mut input = input;
        let mut res = vec![];

        for _ in 0..self.1 {
            if let Some((a, remaining)) = self.0.parse(input) {
                res.push(a);
                input = remaining;
            } else {
                return None;
            }
        }

        Some((res, input))
    }
}

impl<P> Parser<String> for Count<P>
where
    P: Parser<char>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(String, &'a str)> {
        let mut input = input;
        let mut res = String::new();

        for _ in 0..self.1 {
            if let Some((c, remaining)) = self.0.parse(input) {
                res.push(c);
                input = remaining;
            } else {
                return None;
            }
        }

        Some((res, input))
    }
}

#[derive(Clone)]
pub struct Left<T1, P1, T2, P2>(P1, P2, PhantomData<(T1, T2)>);

impl<T1, P1, T2, P2> Parser<T1> for Left<T1, P1, T2, P2>
where
    P1: Parser<T1>,
    P2: Parser<T2>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T1, &'a str)> {
        if let Some((a, remaining)) = self.0.parse(input) {
            if let Some((_, remaining)) = self.1.parse(remaining) {
                Some((a, remaining))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Right<T1, P1, T2, P2>(P1, P2, PhantomData<(T1, T2)>);

impl<T1, P1, T2, P2> Parser<T2> for Right<T1, P1, T2, P2>
where
    P1: Parser<T1>,
    P2: Parser<T2>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T2, &'a str)> {
        if let Some((_, remaining)) = self.0.parse(input) {
            if let Some((b, remaining)) = self.1.parse(remaining) {
                Some((b, remaining))
            } else {
                None
            }
        } else {
            None
        }
    }
}

// impl<T> Parser<T> for Box<dyn Parser<T>> {
//     fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
//         self.parse(input)
//     }
// }

// impl<T> Parser<T> for std::rc::Rc<dyn Parser<T>> {
//     fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
//         self.parse(input)
//     }
// }

// impl<T> Parser<T> for std::sync::Arc<dyn Parser<T>> {
//     fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
//         self.parse(input)
//     }
// }
// 这3个impl根本不用写，自动的

#[derive(Clone)]
pub struct Function<F>(F);

impl<T, F> Parser<T> for Function<F>
where
    // F: for<'r> Fn(&'r str) -> Option<(T, &'r str)>, // 这个for<'r>是什么意思？
    F: Fn(&str) -> Option<(T, &str)>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
        (self.0)(input)
    }
}

// 因为Function没有限定f的类型，而Rust里无法给closure指定lifetime，有可能遇到Function(...)没有实现Parser的问题，所以这里用一个函数限定一下，其实我是很想直接impl<T, F> Parser for F where F: Fn(&str) -> Option<(T, &str)>的
// https://stackoverflow.com/questions/31362206/expected-bound-lifetime-parameter-found-concrete-lifetime-e0271/31365625#31365625 这里还提到了用带输入类型限定的dummy函数来间接给closure标记lifetime的方法
pub fn function<T, F>(f: F) -> Function<F>
where
    F: Fn(&str) -> Option<(T, &str)>,
{
    Function(f)
}

// 梦想终于实现了！
impl<T, F> Parser<T> for F
where
    F: Fn(&str) -> Option<(T, &str)>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
        (self)(input)
    }
}

// impl<T, P> Fn(&str) -> Option<(T, &str)> for P where P: Parser<T> {}
// 不过暂时还不能这么做

#[derive(Clone)]
pub struct LookAhead<P>(P);

impl<T, P> Parser<T> for LookAhead<P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
        if let Some((a, _)) = self.0.parse(input) {
            Some((a, input))
        } else {
            None
        }
    }
}

pub fn symbol(s: &str) -> impl Parser<&str> + Clone {
    Str(s).left(whitespaces)
}
// 觉得这个好像没什么用orz

// 不限定parse出来是u64或者其他类型，方便和无限位精度库梦幻联动
pub fn decimal<T, E>(input: &str) -> Option<(T, &str)>
where
    T: FromStr<Err = E>,
    E: Debug, // 这好烦
{
    digit
        .many1()
        .map(|v: String| v.parse().unwrap())
        .parse(input) // ...many1()之后无法确定是Parser<String>还是Parser<Vec<T>>，可以用map强行让编译器推断出前面是Parser<String>
}
// 比如rug的无限精度Integer也实现了FromStr，所以可以直接parse出这个

pub fn sign(s: &str) -> Option<(char, &str)> {
    Choice(Char('+'), Char('-')).parse(s)
}

pub fn integer<T, E>(input: &str) -> Option<(T, &str)>
where
    T: FromStr<Err = E>,
    E: Debug,
{
    let (sign_, input) = sign.left(whitespaces).parse(input).unwrap_or(('+', input));
    let (digits, input) = digit.many1().map(|v: String| v).parse(input)?;
    if let Ok(v) = format!("{}{}", sign_, digits).parse::<T>() {
        Some((v, input))
    } else {
        None
    }
}
// Haskell parsec的integer是lexeme的，而且可以parse十六进制

#[derive(Clone)]
pub struct SeparatedBy<T1, P1, T2, P2>(P1, P2, PhantomData<(T1, T2)>);

// 写的太难看了……
impl<T1, P1, T2, P2> Parser<Vec<T1>> for SeparatedBy<T1, P1, T2, P2>
where
    P1: Parser<T1>,
    P2: Parser<T2>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(Vec<T1>, &'a str)> {
        let mut input = input;
        let mut res = vec![];

        // 先尝试parse第一个元素
        if let Some((v, remaining)) = self.0.parse(input) {
            res.push(v);
            input = remaining;
        } else {
            return Some((res, input));
        }

        loop {
            // 然后parse分隔符、元素、分隔符、元素……
            if let Some((_, tail1)) = self.1.parse(input) {
                // tail1是吃掉分隔符之后的输入
                if let Some((v, tail2)) = self.0.parse(tail1) {
                    // tail2是吃掉元素之后的输入
                    // 一定要分隔符、元素都成功了，这块才算结束
                    res.push(v);
                    input = tail2
                } else {
                    break Some((res, input)); // 一旦不成功就把input回退到parse分隔符之前的样子
                }
            } else {
                break Some((res, input));
            }
        }
    }
}

#[derive(Clone)]
pub struct Optional<T, P>(P, PhantomData<T>);

impl<T, P> Parser<()> for Optional<T, P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<((), &'a str)> {
        if let Some((_, remaining)) = self.0.parse(input) {
            Some(((), remaining))
        } else {
            Some(((), input))
        }
    }
}

#[derive(Clone)]
pub struct SeparatedEndBy<T1, P1, T2, P2>(P1, P2, PhantomData<(T1, T2)>);

impl<T1, P1, T2, P2> Parser<Vec<T1>> for SeparatedEndBy<T1, P1, T2, P2>
where
    P1: Parser<T1>,
    P2: Parser<T2>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(Vec<T1>, &'a str)> {
        // 为什么不能在内部临时建parser然后直接用呢？
        let mut input = input;
        let mut res = vec![];

        if let Some((v, remaining)) = self.0.parse(input) {
            res.push(v);
            input = remaining;
        } else {
            return Some((res, input));
        }

        loop {
            if let Some((_, tail1)) = self.1.parse(input) {
                if let Some((v, tail2)) = self.0.parse(tail1) {
                    res.push(v);
                    input = tail2
                } else {
                    break Some((res, tail1)); // 和SeparatedBy只有一个单词的区别。分隔符parse成功但元素不成功，不需要把input回退到parse分隔符之前的样子
                }
            } else {
                break Some((res, input));
            }
        }
    }
}

#[derive(Clone)]
pub struct ChainLeft1<T1, P1, T2, P2>(P1, P2, PhantomData<(T1, T2)>);

impl<T1, P1, T2, P2> Parser<T1> for ChainLeft1<T1, P1, T2, P2>
where
    P1: Parser<T1>,
    P2: Parser<T2>,
    T2: Fn(T1, T1) -> T1,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T1, &'a str)> {
        if let Some((acc, remaining)) = self.0.parse(input) {
            let mut acc = acc;
            let mut input = remaining;

            loop {
                if let Some((f, tail1)) = self.1.parse(input) {
                    if let Some((w, tail2)) = self.0.parse(tail1) {
                        input = tail2;
                        acc = f(acc, w);
                    } else {
                        break Some((acc, input));
                    }
                } else {
                    break Some((acc, input));
                }
            }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct ChainRight1<T1, P1, T2, P2>(P1, P2, PhantomData<(T1, T2)>);

impl<T1, P1, T2, P2> Parser<T1> for ChainRight1<T1, P1, T2, P2>
where
    P1: Parser<T1>,
    P2: Parser<T2>,
    T2: Fn(T1, T1) -> T1,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T1, &'a str)> {
        if let Some((v, tail1)) = self.0.parse(input) {
            if let Some((f, tail2)) = self.1.parse(tail1) {
                if let Some((w, tail3)) = self.parse(tail2) {
                    Some((f(v, w), tail3))
                } else {
                    Some((v, tail1))
                }
            } else {
                Some((v, tail1))
            }
        } else {
            None
        }
    }
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
        let parser = satisfy(|c| c.is_ascii_uppercase());
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
        let parser = satisfy(|c| c.is_digit(10));
        assert_eq!(dbg!(parser.parse(input)), Some(('1', "bc")));
    }

    #[test]
    fn satisfy_parse_non_digit() {
        let input = "abc";
        let parser = satisfy(|c| c.is_digit(10));
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
    fn none_of_parse_succeed() {
        let input = "c";
        let parser = none_of(&['x', 'y', 'z']);
        assert_eq!(dbg!(parser.parse(input)), Some(('c', "")));
    }

    #[test]
    fn none_of_parse_fail() {
        let input = "c";
        let parser = none_of(&['a', 'b', 'c']);
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
        let parser = digit.many();
        assert_eq!(dbg!(parser.parse(input)), Some(("1234".to_owned(), "")));
    }

    #[test]
    fn many_parse_number_prefix() {
        let input = "1234abc";
        let parser = digit.many();
        assert_eq!(dbg!(parser.parse(input)), Some(("1234".to_owned(), "abc"))); // 很神奇，这里明明会有歧义，parser究竟是Parser<String>还是Parser<Vec<char>>
        assert_eq!(
            dbg!(Parser::<Vec<char>>::parse(&parser, input)),
            Some(("1234".chars().collect(), "abc"))
        ); // 哈哈，两种都是
    }

    #[test]
    fn many_parse_number_fail() {
        let input = "abc";
        let parser = digit.many();
        assert_eq!(dbg!(parser.parse(input)), Some(("".to_owned(), "abc")));
    }

    #[test]
    fn many1_parse_number() {
        let input = "1abc";
        let parser = digit.many1();
        assert_eq!(dbg!(parser.parse(input)), Some(("1".to_owned(), "abc")));
    }

    #[test]
    fn many1_parse_number_fail() {
        let input = "abc";
        let parser = digit.many1();
        assert_eq!(dbg!(Parser::<String>::parse(&parser, input)), None);
        // assert_eq!(
        //     dbg!(parser
        //         .clone() // 很可惜，Fn不满足Clone
        //         .map(|v: Vec<char>| v.into_iter().collect::<String>())
        //         .parse(input)),
        //     None
        // ); // 或许可以把parser包装在Arc里，就能clone了
        assert_eq!(dbg!(parser.map(|v: String| v).parse(input)), None); // 或者也可以这样写，强行让它通过后面的map认为前面是Parser<String>
    }

    #[test]
    fn choice_parse_alpha_numeric() {
        let input = "a1b2我c3";
        let alpha = satisfy(|c| c.is_ascii_alphabetic());
        let parser = alpha.choice(digit).many();
        assert_eq!(dbg!(parser.parse(input)), Some(("a1b2".to_owned(), "我c3")));
    }

    #[test]
    fn choice_parse_common_prefix() {
        let input = "cat";
        let parser = string("camel").choice(string("cat")).choice(string("dog"));
        assert_eq!(dbg!(parser.parse(input)), Some(("cat", "")));
    }

    #[test]
    fn whitespace_succeed() {
        let input = "   abc";
        let parser = whitespaces;
        assert_eq!(dbg!(parser.parse(input)), Some(((), "abc")));
    }

    #[test]
    fn and_then() {
        let input = "0a1b0a1b0b";
        let parser = satisfy(|c| c.is_digit(10))
            .map(|c| match c {
                '0' => 0,
                _ => 1,
            }) // Parser<i32>
            .and_then(|v| {
                if v == 0 {
                    satisfy({ |c| c == 'a' } as fn(char) -> bool)
                } else {
                    satisfy({ |c| c == 'b' } as fn(char) -> bool)
                } // 这里为了体现closure是单例的，两个closure即使定义完全一样，也被认为是两种类型，if-else的两个臂不能是不同的类型，所以这里只能要么包装成trait object（然后又会有Box<dyn Trait> does not implement Trait的问题）、要么像这样把不捕获环境的closure强行转换成函数指针（reference里说这应该是自动的……）
            }) // Parser<char>
            .many() // Parser<String>或者Parser<Vec<char>>
            .map(|v: String| v);
        assert_eq!(dbg!(parser.parse(input)), Some(("abab".to_owned(), "0b"))); // 因为Some里面是String，所以上面推断出是Parser<String>
    }

    #[test]
    fn count_succeed() {
        let input = "12345";
        let parser = satisfy(|c| c.is_digit(10)).count(5);
        assert_eq!(dbg!(parser.parse(input)), Some(("12345".to_owned(), "")));
    }

    #[test]
    fn count_fail() {
        let input = "1234";
        let parser = satisfy(|c| c.is_digit(10)).count(5);
        assert_eq!(dbg!(Parser::<String>::parse(&parser, input)), None); // 可以这样写
        assert_eq!(dbg!(parser.map(|v: String| v).parse(input)), None); // 也可以这样写
    }

    #[test]
    fn parentheses_surrounding_digits() {
        let input = "(1234)";
        let parser = char('(')
            .right(satisfy(|c| c.is_digit(10)).many())
            .left(char(')'));
        assert_eq!(dbg!(parser.parse(input)), Some(("1234".to_owned(), "")));
    }

    #[test]
    fn digits_between_letters() {
        let input = "abba12234xyzz";
        let letters = satisfy(|c| c.is_ascii_alphabetic())
            .many()
            .map(|v: String| v);
        let digits = satisfy(|c| c.is_digit(10)).many().map(|v: String| v);
        let parser = digits.between(letters.clone(), letters);
        assert_eq!(dbg!(parser.parse(input)), Some(("12234".to_owned(), "")));
    }

    #[test]
    fn look_ahead_succeed() {
        let input = "1234";
        let parser = char('1').look_ahead();
        assert_eq!(dbg!(parser.parse(input)), Some(('1', "1234")));
    }

    #[test]
    fn look_ahead_fail() {
        let input = "1234";
        let parser = char('4').look_ahead();
        assert_eq!(dbg!(parser.parse(input)), None);
    }

    #[test]
    fn epsilon_empty_string() {
        let input = "";
        let parser = epsilon;
        assert_eq!(dbg!(parser.parse(input)), Some(((), "")));
    }

    #[test]
    fn epsilon_non_empty_string() {
        let input = "123";
        let parser = epsilon;
        assert_eq!(dbg!(parser.parse(input)), Some(((), "123")));
    }

    #[test]
    fn closure_parser() {
        let parser: fn(&str) -> Option<(char, &str)> = |input: &str| -> Option<(char, &str)> {
            if let Some(c) = input.chars().next() {
                Some((c, input))
            } else {
                None
            }
        }; // let parser = char('a').look_ahead();

        // 或者套一个function在外面
        // let parser = function(|input: &str| -> Option<(char, &str)> {
        //     if let Some(c) = input.chars().next() {
        //         Some((c, input))
        //     } else {
        //         None
        //     }
        // });
        assert_eq!(dbg!(parser.parse("abc")), Some(('a', "abc")));
    }

    #[test]
    fn valid_parentheses() {
        // 想要构造类似e := "(" e ")" | ""这样的递归文法，可能一开始会想到let parser = eof().choice(char('(').right(parser).left(char(')')))，可是Rust里变量无法self reference
        // 那么什么东西可以self reference呢？我所知道的唯一能自指的东西是函数
        // 正好读到用Go写的parser combinator的一篇文章 https://medium.com/@armin.heller/parser-combinator-gotchas-2792deac4531

        // 文法是这样的
        // s := "" | p*
        // p := "()" | "(" s ")"
        // 但其实也有区别，因为choice是有顺序的
        fn s(input: &str) -> Option<((), &str)> {
            eof.choice(
                string("()")
                    .map(|_| ())
                    .choice(s.between(char('('), char(')')))
                    .many()
                    .map(|_| ()),
            )
            .parse(input)
        }
        // 很想把Function(s)外面的Function去掉，直接让Fn(&str) -> Option<(T, &a)>也实现Parser<T>
        // 实现了

        let parser = s.right(eof); // 加一个right(eof())是为了识别整个字符串。如果parse之后还剩下一段字符串，不算是valid parentheses的
        assert_eq!(dbg!(parser.parse("")), Some(((), "")));
        assert_eq!(dbg!(parser.parse("((()))")), Some(((), "")));
        assert_eq!(dbg!(parser.parse("((()))()")), Some(((), "")));
        assert_eq!(dbg!(parser.parse("((()))(()(()))")), Some(((), "")));
        assert_eq!(dbg!(parser.parse("((()))(()(())()()())")), Some(((), "")));
        assert_eq!(
            dbg!(parser.parse("((()))(()(()))".repeat(1000).as_str())),
            Some(((), ""))
        );
        assert_eq!(dbg!(parser.parse(")((")), None);
        assert_eq!(dbg!(parser.parse("((())")), None);
        assert_eq!(dbg!(parser.parse("(()")), None);
    }

    #[test]
    fn parse_decimal() {
        let parser = decimal;
        assert_eq!(dbg!(parser.parse("1234")), Some((1234, "")));
        assert_eq!(dbg!(parser.parse("0")), Some((0, "")));
        assert_eq!(dbg!(parser.parse("1234abc")), Some((1234, "abc")));
        assert_eq!(dbg!(parser.parse("1 23")), Some((1, " 23")));
    }

    #[test]
    fn parse_integer() {
        let parser = integer.lexeme();
        assert_eq!(dbg!(parser.parse("+1234")), Some((1234, "")));
        assert_eq!(dbg!(parser.parse("+  1234")), Some((1234, "")));
        assert_eq!(dbg!(parser.parse("1234  ")), Some((1234, "")));
        assert_eq!(dbg!(parser.parse("-1234")), Some((-1234, "")));
        assert_eq!(dbg!(parser.parse("-  1234")), Some((-1234, "")));
        assert_eq!(dbg!(parser.parse("12 34")), Some((12, "34")));
        assert_eq!(dbg!(parser.parse("+ 12 34")), Some((12, "34")));
        assert_eq!(dbg!(parser.parse("- 12 34")), Some((-12, "34")));
        assert_eq!(dbg!(parser.parse("a12 34")), None);
    }

    #[test]
    fn comma_separated_integers() {
        let parser = integer.lexeme().separated_by(char(',').lexeme());
        assert_eq!(dbg!(parser.parse("1,2,3")), Some((vec![1, 2, 3], "")));
        assert_eq!(dbg!(parser.parse("1,2,3,")), Some((vec![1, 2, 3], ",")));
        assert_eq!(dbg!(parser.parse("+1, -2, 3")), Some((vec![1, -2, 3], "")));
        assert_eq!(
            dbg!(parser.parse("+ 1 , - 2 , 3 ")),
            Some((vec![1, -2, 3], ""))
        );
    }

    #[test]
    fn rust_vector_macro() {
        // let list = integer()
        //     .lexeme()
        //     .separated_by(char(',').lexeme())
        //     .left(char(',').optional());
        let list = integer.lexeme().separated_end_by(char(',').lexeme());
        let parser = list.between(string("vec![").lexeme(), string("]"));
        assert_eq!(dbg!(parser.parse("vec![1,2,3]")), Some((vec![1, 2, 3], "")));
        assert_eq!(
            dbg!(parser.parse("vec![ +1, -2, 3,]")),
            Some((vec![1, -2, 3], ""))
        );
        assert_eq!(
            dbg!(parser.parse("vec![ 1 , 2 , 3 ]")),
            Some((vec![1, 2, 3], ""))
        );
        assert_eq!(
            dbg!(parser.parse("vec![ 1 , 2 , 3 , ]")),
            Some((vec![1, 2, 3], ""))
        );
    }

    #[test]
    fn evaluate() {
        let number = integer.lexeme().map(|v: i64| v);

        fn add(v: i64, w: i64) -> i64 {
            v + w
        }

        fn subtract(v: i64, w: i64) -> i64 {
            v - w
        }

        let operator = char('+')
            .lexeme()
            .map(|_| add as fn(i64, i64) -> i64) // 这里必须要写as，因为add其实不是函数指针fn，而是一个叫做fn item的东西。和closure一样，fn item也是单例的，每个fn item即使签名相同、也是完全不同的类型，所以一样要cast到函数指针
            .choice(char('-').lexeme().map(|_| subtract as fn(i64, i64) -> i64));
        // https://stackoverflow.com/questions/27895946/expected-fn-item-found-a-different-fn-item-when-working-with-function-pointer
        let parser = number.clone().chain_left1(operator.clone());
        assert_eq!(dbg!(parser.parse("+1")), Some((1, "")));
        assert_eq!(dbg!(parser.parse("+1+2-3")), Some((0, "")));
        assert_eq!(dbg!(parser.parse("+ 1 + 2 - 3")), Some((0, "")));
        assert_eq!(dbg!(parser.parse("- 1 + - 2 - + 3")), Some((-6, "")));
        assert_eq!(dbg!(parser.parse("+ 1 +-+3")), Some((1, "+-+3")));

        // 让加减号变成右结合
        let parser = number.clone().chain_right1(operator.clone());
        assert_eq!(dbg!(parser.parse("-1-2-3")), Some((0, ""))); // (-1) - ((-2) - 3)
    }

    // 更复杂的全功能计算器在examples/arithmetic.rs里
}
