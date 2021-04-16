use parsec::char;
use parsec::function;
use parsec::integer;
use parsec::Parser;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expression {
    Number(i64),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
}

fn add(v: Expression, w: Expression) -> Expression {
    Expression::Add(v.into(), w.into())
} // 这样真的好难看……如果能直接在map里传入variant constructor就好了，就很优雅，而且Rust里的enum variant确实是个函数。但是这样没法处理Box。

fn subtract(v: Expression, w: Expression) -> Expression {
    Expression::Subtract(v.into(), w.into())
}

fn multiply(v: Expression, w: Expression) -> Expression {
    Expression::Multiply(v.into(), w.into())
}

fn divide(v: Expression, w: Expression) -> Expression {
    Expression::Divide(v.into(), w.into())
}

// 按照自然思路会写出这样的文法
// operator := "+"
//     | "-"
// expression := integer
//     | "(" expression ")"
//     | expression operator expression ... expression
// 然后写出这样的parser
// fn expression(input: &str) -> Option<(Expression, &str)> {
//     let operator = char('+')
//         .map(|_| add as fn(Expression, Expression) -> Expression)
//         .choice(char('-').map(|_| subtract as fn(Expression, Expression) -> Expression))
//         .lexeme();
//     let rule1 = integer().lexeme().map(Expression::Number);
//     let rule2 = function(expression).between(char('(').lexeme(), char(')').lexeme());
//     let rule3 = function(expression).chain_left1(operator);
//     rule1.choice(rule2).choice(rule3).parse(input)
// }
// 很可惜，这样的parser无法停止，会无限递归，不知道为啥

// operator := "+"
//     | "-"
// expression := term operator term ... term
// fn expression(input: &str) -> Option<(Expression, &str)> {
//     let operator = char('+')
//         .lexeme()
//         .map(|_| add as fn(Expression, Expression) -> Expression)
//         .choice(
//             char('-')
//                 .lexeme()
//                 .map(|_| subtract as fn(Expression, Expression) -> Expression),
//         );
//     function(term).chain_left1(operator).parse(input)
// }

// 下面用来处理运算符优先级的做法来自 https://stackoverflow.com/questions/56295777/right-way-to-parse-chain-of-various-binary-functions-with-parsec
// 我也没理解……先拿来用再说

// 低优先级运算，比如加减
fn level0(input: &str) -> Option<(Expression, &str)> {
    let operator = char('+')
        .lexeme()
        .map(|_| add as fn(Expression, Expression) -> Expression)
        .choice(
            char('-')
                .lexeme()
                .map(|_| subtract as fn(Expression, Expression) -> Expression),
        );
    function(level1).chain_left1(operator).parse(input)
}

// 高优先级运算符，比如乘除
fn level1(input: &str) -> Option<(Expression, &str)> {
    let operator = char('*')
        .lexeme()
        .map(|_| multiply as fn(Expression, Expression) -> Expression)
        .choice(
            char('/')
                .lexeme()
                .map(|_| divide as fn(Expression, Expression) -> Expression),
        );
    function(term).chain_left1(operator).parse(input)
}

// term := integer
//     | "(" expression ")"
fn term(input: &str) -> Option<(Expression, &str)> {
    integer()
        .lexeme()
        .map(Expression::Number)
        .choice(function(level0).between(char('(').lexeme(), char(')').lexeme()))
        .parse(input)
}

impl Expression {
    pub fn evaluate(&self) -> i64 {
        match self {
            Self::Number(v) => *v,
            Self::Add(v, w) => v.evaluate() + w.evaluate(),
            Self::Subtract(v, w) => v.evaluate() - w.evaluate(),
            Self::Multiply(v, w) => v.evaluate() * w.evaluate(),
            Self::Divide(v, w) => v.evaluate() / w.evaluate(),
        }
    }
}

fn main() {
    let parser = function(level0);

    assert_eq!(
        dbg!(parser.parse("(+1+2)-3")),
        Some((
            Expression::Subtract(
                Expression::Add(Expression::Number(1).into(), Expression::Number(2).into(),).into(),
                Expression::Number(3).into(),
            ),
            ""
        ))
    );
    assert_eq!(
        dbg!(parser.parse("+1-(-2-3)")),
        Some((
            Expression::Subtract(
                Expression::Number(1).into(),
                Expression::Subtract(Expression::Number(-2).into(), Expression::Number(3).into(),)
                    .into(),
            ),
            ""
        ))
    );
    assert_eq!(
        dbg!(parser.parse("1 + 2 * 3")),
        Some((
            Expression::Add(
                Expression::Number(1).into(),
                Expression::Multiply(Expression::Number(2).into(), Expression::Number(3).into(),)
                    .into()
            ),
            ""
        ))
    );
    assert_eq!(
        dbg!(parser.parse("(1 + 2) * 3")),
        Some((
            Expression::Multiply(
                Expression::Add(Expression::Number(1).into(), Expression::Number(2).into(),).into(),
                Expression::Number(3).into(),
            ),
            ""
        ))
    );

    let calculator = parser.map(|v| v.evaluate());

    assert_eq!(dbg!(calculator.parse("(1 + 2) * 3")), Some((9, "")));
    assert_eq!(dbg!(calculator.parse("1 + 2 * 3")), Some((7, "")));
}
