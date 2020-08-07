use super::{
    BinaryOp, Expression, FunctionDefinition, Identifier, Number, Statement, UnaryOp,
    VariableAssignment,
};
use combine::easy::{self, Error};
use combine::parser::char::{alpha_num, char, crlf, digit, letter, newline, string};
use combine::parser::combinator::recognize;
use combine::ParseError;
use combine::{
    attempt, between, choice, eof, many, one_of, optional, parser, satisfy, sep_by, skip_many,
    skip_many1, EasyParser, Parser, Stream,
};
use itertools::Itertools;

pub fn parse(input: &str) -> Result<Vec<Statement>, easy::Errors<char, &str, usize>> {
    let mut script = lex(stmt_list()).skip(eof());

    script
        .easy_parse(input)
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
    let comment = || (char('#'), skip_many(satisfy(|c| c != '\n' && c != '\r')));
    let nl = || newline().or(attempt(crlf()));

    // map(|_| ()) is just for matching types
    sep_by(
        optional(spaces()).with(lex(optional(stmt()))),
        choice((
            lex(char(';')).map(|_| ()),
            attempt(nl()).map(|_| ()),
            attempt(comment().and(nl())).map(|_| ()),
        )),
    )
    .skip(optional(choice((
        lex(char(';')).map(|_| ()),
        attempt(comment()).map(|_| ()),
    ))))
    .map(|maybe_stmts: Vec<_>| {
        maybe_stmts
            .iter()
            .filter_map(|x| x.as_ref())
            .cloned()
            .collect()
    })
}

fn stmt<I>() -> impl Parser<I, Output = Statement>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    lex(choice((
        attempt(def_func().map(Statement::FunctionDefinition)),
        attempt(assign_var().map(Statement::VariableAssignment)),
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

fn assign_var<I>() -> impl Parser<I, Output = VariableAssignment>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    (ident(), lex(char('=')), expr())
        .map(|a| VariableAssignment {
            name: a.0,
            expr: a.2,
        })
        .expected("variable assignment")
}

fn def_func<I>() -> impl Parser<I, Output = FunctionDefinition>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    let func = ident()
        .and(between(
            lex(char('(')),
            lex(char(')')),
            sep_by(lex(ident()), lex(char(','))),
        ))
        .expected("function");

    (func, lex(char('=')), expr())
        .map(|((name, arg_names), _, expr)| FunctionDefinition {
            name,
            arg_names,
            expr,
        })
        .expected("function definition")
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
        .map(|(lhs, rhs): (_, Vec<_>)| {
            rhs.into_iter().fold(lhs, |a, (op, b)| {
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
        .map(|(lhs, rhs): (_, Vec<_>)| {
            rhs.into_iter().fold(lhs, |a, (op, b)| {
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
        .map(|(lhs, rhs): (_, Vec<_>)| {
            rhs.into_iter().fold(lhs, |a, b| {
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
        .map(|(lhs, rhs): (_, Vec<_>)| {
            // power is right associative
            std::iter::once(lhs)
                .chain(rhs.into_iter())
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
            number().map(Expression::Number),
            attempt(apply_func()),
            ident().map(Expression::Variable),
        ))
    }
}

fn parens<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    between(lex(char('(')), lex(char(')')), lex(expr())).expected("parentheses")
}

fn apply_func<I>() -> impl Parser<I, Output = Expression>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
    ident()
        .and(between(
            lex(char('(')),
            lex(char(')')),
            sep_by(lex(expr()), lex(char(','))),
        ))
        .map(|(name, args)| Expression::Function(name, args))
        .expected("function")
}

fn number<I>() -> impl Parser<I, Output = Number>
where
    I: Stream<Token = char, Error = easy::ParseError<I>>,
    I::Range: PartialEq,
    I::Error: ParseError<I::Token, I::Range, I::Position, StreamError = Error<I::Token, I::Range>>,
{
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

fn spaces<I>() -> impl Parser<I, Output = ()>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let space =
        satisfy(|c: char| c != '\n' && c != '\r' && c.is_whitespace()).expected("whitespace");
    skip_many(space).expected("whitespaces")
}
