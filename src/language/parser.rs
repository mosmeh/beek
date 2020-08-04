use super::*;
use combine::easy::{self, Error};
use combine::parser::char::{alpha_num, char, digit, letter, spaces, string};
use combine::parser::combinator::recognize;
use combine::ParseError;
use combine::{
    any, attempt, choice, eof, many, one_of, optional, parser, skip_many, skip_many1, EasyParser,
    Parser, Stream,
};
use itertools::Itertools;

pub fn parse(input: &str) -> Result<Vec<Statement>, easy::Errors<char, &str, usize>> {
    let comment = (char('#'), skip_many(any()));
    let mut code = optional(spaces())
        .with(lex(stmt_list()))
        .skip(optional(comment))
        .skip(eof());

    code.easy_parse(input)
        .map(|(parsed, rem)| {
            assert!(rem.is_empty());
            parsed
        })
        .map_err(|err| err.map_position(|p| p.translate_position(input)))
}

fn stmt_list<I>() -> impl Parser<I, Output = Vec<Statement>>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    let semicolons = || skip_many(lex(char(';')));
    optional(semicolons()).with(lex(many(lex(stmt()).skip(semicolons()))))
}

fn stmt<I>() -> impl Parser<I, Output = Statement>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    lex(choice((
        attempt(immediate_assign().map(Statement::ImmediateAssignment)),
        attempt(lazy_assign().map(Statement::LazyAssignment)),
        expr().map(Statement::Expression),
    )))
    .expected("statement")
}

fn expr<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    expr_()
}

parser! {
    fn expr_[I]()(I) -> Expression
    where [
        I: Stream<Token = char, Error = easy::ParseError<I>>,
        I::Range: PartialEq,
        I::Error:
            ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
    ]
    {
        lex(add()).expected("expression")
    }
}

fn lazy_assign<I>() -> impl Parser<I, Output = Assignment>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    (ident(), lex(char('=')), expr())
        .map(|a| Assignment {
            var: a.0,
            expr: a.2,
        })
        .expected("assignment")
}

fn immediate_assign<I>() -> impl Parser<I, Output = Assignment>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    (ident(), lex(string(":=")), expr())
        .map(|a| Assignment {
            var: a.0,
            expr: a.2,
        })
        .expected("assignment")
}

fn add<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    mul()
        .and(many(
            lex(char('+')
                .map(|_| BinaryOp::Add)
                .or(char('-').map(|_| BinaryOp::Subtract)))
            .and(mul()),
        ))
        .map(|(expr, vec): (_, Vec<_>)| {
            vec.into_iter().fold(expr, |a, (op, b)| {
                Expression::BinaryOp(op, Box::new(a), Box::new(b))
            })
        })
}

fn mul<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    negate()
        .and(many(
            lex(choice((char('*'), char('·'), char('×')))
                .map(|_| BinaryOp::Multiply)
                .or(char('/').or(char('÷')).map(|_| BinaryOp::Divide))
                .or(char('%').map(|_| BinaryOp::Modulo)))
            .and(negate()),
        ))
        .map(|(expr, vec): (_, Vec<_>)| {
            vec.into_iter().fold(expr, |a, (op, b)| {
                Expression::BinaryOp(op, Box::new(a), Box::new(b))
            })
        })
}

fn negate<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    lex((optional(lex(sign())), implicit_mul())).map(|(sign, expr)| {
        if let Some('-') = sign {
            Expression::UnaryOp(UnaryOp::Negate, Box::new(expr))
        } else {
            expr
        }
    })
}

fn implicit_mul<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    exp()
        .and(many(spaces().with(exp())))
        .map(|(expr, vec): (_, Vec<_>)| {
            vec.into_iter().fold(expr, |a, b| {
                Expression::BinaryOp(BinaryOp::Multiply, Box::new(a), Box::new(b))
            })
        })
}

fn exp<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    factorial()
        .and(many(
            lex(string("^").or(attempt(string("**")))).with(factorial()),
        ))
        .map(|(expr, vec): (_, Vec<_>)| {
            // power is right associative
            std::iter::once(expr)
                .chain(vec.into_iter())
                .rev()
                .fold1(|a, b| Expression::BinaryOp(BinaryOp::Power, Box::new(b), Box::new(a)))
                .unwrap()
        })
}

fn factorial<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    lex((atom(), optional(char('!'))).map(|(atom, fact)| {
        if fact.is_some() {
            Expression::UnaryOp(UnaryOp::Factorial, Box::new(atom))
        } else {
            atom
        }
    }))
}

fn atom<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    atom_()
}

parser! {
    fn atom_[I]()(I) -> Expression
    where [
        I: Stream<Token = char, Error = easy::ParseError<I>>,
        I::Range: PartialEq,
        I::Error:
            ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
    ]
    {
        choice((
            parens(),
            ident().map(Expression::Identifier),
            number().map(Expression::Number),
        ))
    }
}

fn parens<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    lex(char('(')).with(lex(expr())).skip(lex(char(')')))
}

fn number<I>() -> impl Parser<I, Output = Number>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    // map(|_| ()) is just for matching types
    let without_int_part = (char('.'), skip_many1(digit())).map(|_| ());
    let with_int_part = (
        skip_many1(digit()),
        optional((char('.'), skip_many(digit()))),
    )
        .map(|_| ());
    let mantissa = without_int_part.or(with_int_part);

    let exponent = (one_of("eE".chars()), optional(sign()), skip_many1(digit()));

    lex(recognize((mantissa, optional(exponent))).map(|x: String| Number(x.parse().unwrap())))
        .expected("number")
}

fn ident<I>() -> impl Parser<I, Output = Identifier>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    let ident_start = letter().or(char('_'));
    let ident_letter = alpha_num().or(char('_'));

    lex(recognize((ident_start, skip_many(ident_letter))))
        .map(Identifier)
        .expected("identifier")
}

fn lex<I, P>(p: P) -> impl Parser<I, Output = P::Output>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
    P: Parser<I>,
{
    p.skip(spaces())
}

fn sign<I>() -> impl Parser<I, Output = char>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    one_of("-+".chars())
}
