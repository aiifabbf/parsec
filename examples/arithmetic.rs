use parsec::char;
use parsec::function;
use parsec::integer;
use parsec::Parser;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expression {
    Number(i64),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
}

fn add(v: Expression, w: Expression) -> Expression {
    Expression::Add(v.into(), w.into())
} // 这样真的好难看……如果能直接在map里传入variant constructor就好了，就很优雅，而且Rust里的enum variant确实是个函数。但是这样没法处理Box。

fn subtract(v: Expression, w: Expression) -> Expression {
    Expression::Subtract(v.into(), w.into())
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
fn expression(input: &str) -> Option<(Expression, &str)> {
    let operator = char('+')
        .lexeme()
        .map(|_| add as fn(Expression, Expression) -> Expression)
        .choice(
            char('-')
                .lexeme()
                .map(|_| subtract as fn(Expression, Expression) -> Expression),
        );
    function(term).chain_left1(operator).parse(input)
}

// term := integer
//     | "(" expression ")"
fn term(input: &str) -> Option<(Expression, &str)> {
    integer()
        .lexeme()
        .map(Expression::Number)
        .choice(function(expression).between(char('(').lexeme(), char(')').lexeme()))
        .parse(input)
}

fn main() {
    let parser = function(expression);

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
}
