//! The abstract syntax tree of our little language.
//!
//! This module does the parsing of data into that tree. Evaluation of the
//! tree is then implemented on the structures defined here in the *eval*
//! module.

use std::{borrow, fmt, hash, str};
use std::convert::TryFrom;
use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{
    tag, take_until, take_while, take_while1, take_while_m_n
};
use nom::character::complete::{char as tag_char, multispace1, none_of, one_of};
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::error::ErrorKind;
use nom::multi::{fold_many0, fold_many1, many0, separated_list};
use nom::number::complete::recognize_float;
use nom::sequence::{preceded, terminated, tuple};

type Span<'a> = nom_locate::LocatedSpan<&'a str>;


//============ Statements ====================================================

//------------ StatementList -------------------------------------------------

/// A list of statments.
///
/// ```text
/// statement-list ::= *(statement)
/// ```
///
/// This type also serves as the root of the syntax tree. That is, a map file
/// consists of a list of statements.
#[derive(Clone, Debug)]
pub struct StatementList {
    /// The list of statements.
    pub statements: Vec<Statement>,

    /// The start of the statement list in the source.
    pub pos: Pos,
}

impl StatementList {
    /// Parses a string into a statement list.
    pub fn parse_str(input: &str) -> Result<Self, Error> {
        all_consuming(Self::parse)(Span::new(input))
        .map(|(_, res)| res)
        .map_err(|err| {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => err.into(),
                nom::Err::Incomplete(_) => unreachable!(),
            }
        })
    }

    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, statements) = terminated(
            many0(Statement::parse),
            skip_opt_ws
        )(input)?;
        Ok((input, StatementList { statements, pos }))
    }
}


//------------ Statement -----------------------------------------------------

/// Statements are things that actually do something.
///
/// This type enumerates all currently defined statements.
///
/// ```text
/// statement ::= (contour | label | let | symbol | with)
/// ```
#[derive(Clone, Debug)]
pub enum Statement {
    Contour(Contour),
    Label(Label),
    Let(Let),
    Symbol(Symbol),
    With(With),
}

impl Statement {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(Contour::parse, Statement::Contour),
            map(Label::parse, Statement::Label),
            map(Let::parse, Statement::Let),
            map(Symbol::parse, Statement::Symbol),
            map(With::parse, Statement::With),
        ))(input)
    }
}


//----------- Contour --------------------------------------------------------

/// The coutour statement draws a contour onto the map.
///
/// ```text
/// contour ::= "contour" 
///             [ "with" params:assignment-list ]
///             rule:expression
///             path ";"
/// ```
///
/// If the optional `params` are present, the rendering parameters are updated
/// for the drawing operation.
///
/// The `rule` expression must evaluate to a contour rendering rule. The 
/// contour is then rendered by the rule along the path.
#[derive(Clone, Debug)]
pub struct Contour {
    /// Optional updates to the rendering parameters for this contour.
    pub params: Option<AssignmentList>,

    /// The rendering rule for this contour.
    pub rule: Expression,

    /// The path of the contour.
    pub path: Path,

    /// The start of the contour statement in the source.
    pub pos: Pos
}

impl Contour {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = opt_ws(tag("contour"))(input)?;
        let (input, _) = skip_ws(input)?;
        let (input, params) = opt(
            preceded(
                opt_ws(tag("with")),
                opt_ws(AssignmentList::parse)
            )
        )(input)?;
        let (input, rule) = opt_ws(Expression::parse)(input)?;
        let (input, path) = opt_ws(Path::parse)(input)?;
        let (input, _) = opt_ws(tag_char(';'))(input)?;
        Ok((input, Contour { params, rule, path, pos }))
    }
}


//------------ Label ---------------------------------------------------------

/// The label statement renders text onto the map.
///
/// ```text
/// label ::= "label"
///           [ "with" params:assignment-list ]
///           position
///           layout:expression ";"
/// ```
#[derive(Clone, Debug)]
pub struct Label {
    /// Optional updates to the rendering parameters for this contour.
    pub params: Option<AssignmentList>,

    /// The position of the symbol.
    pub position: Position,

    /// The layout to render.
    pub layout: Expression,

    /// The start of the text statement in the source.
    pub pos: Pos
}

impl Label {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = opt_ws(tag("label"))(input)?;
        let (input, _) = skip_ws(input)?;
        let (input, params) = opt(
            preceded(
                opt_ws(tag("with")),
                opt_ws(AssignmentList::parse)
            )
        )(input)?;
        let (input, position) = opt_ws(Position::parse)(input)?;
        let (input, layout) = opt_ws(Expression::parse)(input)?;
        let (input, _) = opt_ws(tag_char(';'))(input)?;
        Ok((input, Label { params, position, layout, pos }))
    }
}


//----------- Let ------------------------------------------------------------

/// The let statement assigns values to variables;
///
/// ```text
/// let ::= "let" assignment-list ";"
/// ```
///
/// The scope of the variables is the current block or file. One exception is
/// let statements made at file level in a file named “init.map”. These
/// variables are also available in all files in the same directory or below.
///
/// Variables can be overidden by simply defining them anew.
#[derive(Clone, Debug)]
pub struct Let {
    /// The assignments of the let statement.
    pub assignments: AssignmentList,

    /// The start of the let statement in the source.
    pub pos: Pos,
}

impl Let{
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = opt_ws(tag("let"))(input)?;
        let (input, assignments) = ws(AssignmentList::parse)(input)?;
        let (input, _) = opt_ws(tag_char(';'))(input)?;
        Ok((input, Let { assignments, pos }))
    }
}


//------------ Symbol --------------------------------------------------------

/// The symbol statements draw a symbol onto the map.
///
/// ```text
/// symbol ::= "symbol"
///            [ "with" params:assignment-list ]
///            rule:expression
///            position ";"
/// ```
#[derive(Clone, Debug)]
pub struct Symbol {
    /// Optional updates to the rendering parameters for this contour.
    pub params: Option<AssignmentList>,

    /// The rendering rule for this symbol.
    pub rule: Expression,

    /// The position of the symbol.
    pub position: Position,

    /// The start of the symbol statement in the source.
    pub pos: Pos
}

impl Symbol {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = opt_ws(tag("symbol"))(input)?;
        let (input, _) = skip_ws(input)?;
        let (input, params) = opt(
            preceded(
                opt_ws(tag("with")),
                opt_ws(AssignmentList::parse)
            )
        )(input)?;
        let (input, rule) = opt_ws(Expression::parse)(input)?;
        let (input, position) = opt_ws(Position::parse)(input)?;
        let (input, _) = opt_ws(tag_char(';'))(input)?;
        Ok((input, Symbol { params, rule, position, pos }))
    }
}


//----------- With -----------------------------------------------------------

/// The with statement executes a block with updated render parameters.
///
/// ```text
/// with ::= "with" params:assignment-list "{" block:statement-list "}"
/// ```
///
/// Then the with statement is evaluated, a new scope is created as a clone
/// of the current scope. In this scope, the render parameters given in
/// `params` are updated to their evaluated values and then statements in
/// `block` are evaluated for this scope.
#[derive(Clone, Debug)]
pub struct With {
    /// The updates to the rendering parameters.
    pub params: AssignmentList,

    /// The statements to execute.
    pub block: StatementList,

    /// The start of the with statement in the source.
    pub pos: Pos
}

impl With {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = opt_ws(tag("with"))(input)?;
        let (input, params) = ws(AssignmentList::parse)(input)?;
        let (input, _) = opt_ws(tag_char('{'))(input)?;
        let (input, block) = opt_ws(StatementList::parse)(input)?;
        let (input, _) = opt_ws(tag_char('}'))(input)?;
        Ok((input, With { params, block, pos }))
    }
}


//============ Expressions ===================================================

//------------ Expression ----------------------------------------------------

/// An expression.
///
/// Expressions can be evaluated to a value of a certain type. During parsing
/// we accept any kind of expression to keep the AST simple. Type checking is
/// then done during evaluation.
///
/// ```text
/// expression ::= distance | position | range | number | text | function
///              | identifier
/// ```
#[derive(Clone, Debug)]
pub enum Expression {
    Distance(Distance),
    Range(Range),
    Number(Number),
    Text(Text),
    Function(Function),
    Variable(Identifier),
}

impl Expression {
    fn parse(input: Span) -> IResult<Span, Self> {
        // The parsing order here is important since some types include#
        // others. Generally: longer types first.
        alt((
            map(Distance::parse, Expression::Distance),
            map(Range::parse, Expression::Range),
            map(Number::parse, Expression::Number),
            map(Text::parse, Expression::Text),
            map(Function::parse, Expression::Function),
            map(Identifier::parse, Expression::Variable),
        ))(input)
    }

    pub fn pos(&self) -> Pos {
        match *self {
            Expression::Distance(ref expr) => expr.pos,
            Expression::Range(ref expr) => expr.pos,
            Expression::Number(ref expr) => expr.pos,
            Expression::Text(ref expr) => expr.pos,
            Expression::Function(ref expr) => expr.pos,
            Expression::Variable(ref expr) => expr.pos,
        }
    }
}


//------------ Distance ------------------------------------------------------

/// A distance expression is the sum of a number of unit numbers.
///
/// ```text
/// distance ::= unit-number *(["+" | "-"] unit-number)
/// ```
#[derive(Clone, Debug)]
pub struct Distance {
    /// The first element of the distance expression.
    ///
    /// This value is always present.
    pub first: UnitNumber,

    /// The following elements of the distance expression.
    ///
    /// Each element is supposed to be added to or subtracted from the
    /// resulting distance.
    pub others: Vec<(AddSub, UnitNumber)>,

    /// The position of the distance expression in the input.
    pub pos: Pos,
}

impl Distance {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, first) = UnitNumber::parse(input)?;
        let (input, others) = many0(
            tuple((
                opt_ws(AddSub::parse),
                opt_ws(UnitNumber::parse)
            ))
        )(input)?;
        Ok((input, Distance { first, others, pos }))
    }
}


//------------ Range ---------------------------------------------------------

/// A range expression gives an lower and upper bound for a number.
///
/// ```text
/// range ::= number "->" number
/// ```
#[derive(Clone, Debug)]
pub struct Range {
    /// The lower bound of the range.
    pub first: Number,

    /// The upper bound of the range.
    pub second: Number,

    /// The start of the range in the input.
    pub pos: Pos
}

impl Range {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, first) = Number::parse(input)?;
        let (input, _) = opt_ws(tag("->"))(input)?;
        let (input, second) = opt_ws(Number::parse)(input)?;
        Ok((input, Range { first, second, pos }))
    }
}


//------------ Text ----------------------------------------------------------

/// A text expression is a sequence of quoted strings.
///
/// ```text
/// text ::= quoted *(quoted)
/// ```
///
/// The evaluated value of the expression is the concatenation of the quoted
/// strings.
#[derive(Clone, Debug)]
pub struct Text {
    /// The first quoted string of the text expression.
    ///
    /// This string is always present.
    pub first: Quoted,

    /// Any possible additional quoted strings.
    ///
    /// By keeping only additional strings in the vec, we avoid having to
    /// allocate if there aren’t any. 
    pub others: Vec<Quoted>,

    /// The start of the text expression in the input.
    pub pos: Pos
}

impl Text {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, first) = Quoted::parse(input)?;
        let (input, others) = many0(opt_ws(Quoted::parse))(input)?;
        Ok((input, Text { first, others, pos }))
    }
}


//------------ Function ------------------------------------------------------

/// A function expression.
///
/// ```text
/// function ::= name:identifier "(" [argument-list] ")"
/// ```
///
/// The function with `name` is being executed with the evaluated arguments
/// when the function expression is evaluated.
#[derive(Clone, Debug)]
pub struct Function {
    /// The name of the function.
    pub name: Identifier,

    /// The optional arguments of the function.
    pub args: Option<ArgumentList>,

    /// The start of the function expression in the source.
    pub pos: Pos,
}

impl Function {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, name) = Identifier::parse(input)?;
        let (input, _) = opt_ws(tag_char('('))(input)?;
        let (input, args) = opt_ws(opt(ArgumentList::parse))(input)?;
        let (input, _) = opt_ws(tag_char(')'))(input)?;
        Ok((input, Function { name, args, pos }))
    }
}


//============ Path Constructions ============================================


//------------ Path ----------------------------------------------------------

/// A path expression connects expressions with path connectors.
///
/// ```text
/// path ::= sement *(path-connector segment)
/// ```
///
/// When evaluated, the expressions need to evaluate to path segments. These
/// segments are then connected into a path according to the path connectors.
#[derive(Clone, Debug)]
pub struct Path {
    /// The first segment of the path.
    pub first: Segment,

    /// All following segments of the path and how they are connected.
    pub others: Vec<(PathConnector, Segment)>,

    /// The start of the path expression in the source.
    pub pos: Pos,
}

impl Path {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, first) = Segment::parse(input)?;
        let (input, others) = many0(
            tuple((
                opt_ws(PathConnector::parse),
                opt_ws(Segment::parse),
            ))
        )(input)?;
        Ok((input, Path { first, others, pos }))
    }
}


//------------ Segment -------------------------------------------------------

/// A segment expression describes a segment on a path.
///
/// ```text
/// segment ::= path:expression
///             ["[" start:location "," end:location "]" ]
///             [ offset ]
/// ```
#[derive(Clone, Debug)]
pub struct Segment {
    /// The path the segment refers to.
    pub path: Expression,

    /// The optional start and end location.
    pub location: Option<(Location, Location)>,

    /// The optional offset.
    pub offset: Option<Offset>,

    /// The start of the segment in the source
    pub pos: Pos
}

impl Segment {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, path) = Expression::parse(input)?;
        let (input, location) = opt(
            terminated(
                preceded(
                    opt_ws(tag_char('[')),
                    tuple((
                        opt_ws(Location::parse),
                        preceded(
                            opt_ws(tag_char(',')),
                            opt_ws(Location::parse)
                        )
                    ))
                ),
                opt_ws(tag_char(']'))
            )
        )(input)?;
        let (input, offset) = opt(opt_ws(Offset::parse))(input)?;
        Ok((input, Segment { path, location, offset, pos }))
    }
}


//------------ Position ------------------------------------------------------

/// A position expression derives a point and direction from a path.
///
/// ```text
/// position ::= path:expression "[" location "]"
///              [ offset ] [ "@" angle:number ]
/// ```
#[derive(Clone, Debug)]
pub struct Position {
    /// The path the position derives to.
    pub path: Expression,

    /// The location on the path.
    pub location: Location,

    /// The optional offset.
    pub offset: Option<Offset>,

    /// The optional rotation.
    pub rotation: Option<Number>,

    /// The start of the position in the source
    pub pos: Pos
}

impl Position {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, path) = Expression::parse(input)?;
        let (input, location) = terminated(
            preceded(
                opt_ws(tag_char('[')),
                opt_ws(Location::parse)
            ),
            opt_ws(tag_char(']'))
        )(input)?;
        let (input, offset) = opt(opt_ws(Offset::parse))(input)?;
        let (input, rotation) = opt(
            preceded(
                opt_ws(tag_char('@')),
                opt_ws(Number::parse)
            )
        )(input)?;
        Ok((input, Position { path, location, offset, rotation, pos }))
    }
}


//------------ Location ------------------------------------------------------

/// A location on a path.
///
/// ```text
/// location ::= name:identifier [("+" | "-") distance ]
/// ```
#[derive(Clone, Debug)]
pub struct Location {
    /// The name of a point.
    pub name: Identifier,

    /// The optional distance.
    pub distance: Option<(AddSub, Distance)>,

    /// The start of the location in the source.
    pub pos: Pos
}

impl Location {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, name) = Identifier::parse(input)?;
        let (input, distance) = opt(tuple((
            opt_ws(AddSub::parse), opt_ws(Distance::parse)
        )))(input)?;
        Ok((input, Location { name, distance, pos }))
    }
}


//------------ Offset --------------------------------------------------------

/// An offset sideways from a path.
///
/// ```text
/// offset ::= direction distance
/// ```
#[derive(Clone, Debug)]
pub struct Offset {
    /// The direction of the offset.
    pub direction: Direction,

    /// The distance of the offset.
    pub distance: Distance,

    /// The start of the location in the source
    pub pos: Pos,
}

impl Offset {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, direction) = Direction::parse(input)?;
        let (input, distance) = opt_ws(Distance::parse)(input)?;
        Ok((input, Offset { direction, distance, pos }))
    }
}


//------------ Direction -----------------------------------------------------

/// A direction operator indicating left or right.
///
/// ```text
/// direction ::= "<<" | ">>"
/// ```
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(tag("<<"), |_| Direction::Left),
            map(tag(">>"), |_| Direction::Right),
        ))(input)
    }
}


//============ Other Things ==================================================

//------------ AssignmentList ------------------------------------------------

/// An assignment list is a non-empty comma separated list of assignments.
///
/// ```text
/// assignment-list ::= assignment *("," assignment)
/// ```
#[derive(Clone, Debug)]
pub struct AssignmentList {
    /// The assignments of the list.
    pub assignments: Vec<Assignment>,

    /// The start of the assignment list in the source.
    pub pos: Pos,
}

impl AssignmentList {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, assignments) = separated_list(
            opt_ws(tag_char(',')),
            opt_ws(Assignment::parse)
        )(input)?;
        Ok((input, AssignmentList { assignments, pos }))
    }
}


//------------ Assignment ----------------------------------------------------

/// An assignment assigns the result of an expression to a variable.
///
/// ```text
/// assignment ::= target:identifier "=" expression
/// ```
///
/// When the assignment is evaluated, a new variable with the name `target`
/// is added to the current scope. The expression is evaluated at that point
/// already, resolving all variables with the current scope.
#[derive(Clone, Debug)]
pub struct Assignment {
    /// The name of the variable or the keyword.
    pub target: Identifier,

    /// The expression to assign to.
    pub expression: Expression,

    /// The start of the assignment in the source.
    pub pos: Pos,
}

impl Assignment {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, target) = Identifier::parse(input)?;
        let (input, _) = opt_ws(tag_char('='))(input)?;
        let (input, expression) = opt_ws(Expression::parse)(input)?;
        Ok((input, Assignment { target, expression, pos }))
    }
}


//------------ ArgumentList --------------------------------------------------

/// An argument list is a non-empty comma separated list of arguments.
///
/// ```text
/// argument-list ::= argument *("," argument)
/// ```
#[derive(Clone, Debug)]
pub struct ArgumentList {
    /// The list of arguments.
    pub arguments: Vec<Argument>,

    /// The start of the argument list in the source
    pub pos: Pos,
}

impl ArgumentList {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, arguments) = separated_list(
            opt_ws(tag_char(',')),
            opt_ws(Argument::parse)
        )(input)?;
        Ok((input, ArgumentList { arguments, pos }))
    }
}


//------------ Argument ------------------------------------------------------

/// An argument is either a positional or keyword argument.
///
/// ```text
/// argument ::= assignment | expression
/// ```
#[derive(Clone, Debug)]
pub enum Argument {
    Keyword(Assignment),
    Pos(Expression),
}

impl Argument {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(Assignment::parse, Argument::Keyword),
            map(Expression::parse, Argument::Pos),
        ))(input)
    }
}


//============ Tokens ========================================================

//------------ AddSub --------------------------------------------------------

/// Addition or subtraction.
#[derive(Clone, Copy, Debug)]
pub enum AddSub {
    Add,
    Sub
}

impl AddSub {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(tag_char('+'), |_| AddSub::Add),
            map(tag_char('-'), |_| AddSub::Sub)
        ))(input)
    }
}


//------------ Identifier ----------------------------------------------------

/// An identifier is the name of variables or other things.
///
/// It is a word composed of a leading alphabetic Unicode character or an
/// underscore, followed by alphanumeric Unicode characters or underscore or
/// period.
#[derive(Clone, Debug)]
pub struct Identifier {
    /// The actual identifier.
    pub ident: String,

    /// The start of the identifier in the source.
    pub pos: Pos,
}

impl Identifier {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, ident) = recognize(
            preceded(
                take_while1(|ch: char| {
                    ch.is_alphabetic() || ch == '_'
                }),
                take_while(|ch: char| {
                    ch.is_alphanumeric() || ch == '_' || ch == '.'
                })
            )
        )(input)?;
        Ok((input, Identifier { ident: (*ident.fragment()).into(), pos }))
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.ident.as_ref()
    }
}

impl borrow::Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        self.ident.as_ref()
    }
}

impl<T: AsRef<str>> PartialEq<T> for Identifier {
    fn eq(&self, other: &T) -> bool {
        self.ident == other.as_ref()
    }
}

impl Eq for Identifier { }

impl hash::Hash for Identifier {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ident.fmt(f)
    }
}


//------------ Number --------------------------------------------------------

/// A number expression is either a integer number or a floating point number.
///
/// ```text
/// number ::= <integer or floating point number in Rust formatting>
/// ```
///
/// We keep the string representation of the number and only convert upon
/// evaluation.
#[derive(Clone, Debug)]
pub struct Number {
    /// The string representation of the number.
    pub value: String,

    /// The position the number started at.
    pub pos: Pos,
}

impl Number {
    /// Parses the number from an input span.
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, value) = recognize_float(input)?;
        let value = (*value.fragment()).into();
        Ok((input, Number { value, pos }))
    }
}


//------------ PathConnector -------------------------------------------------

/// A path connector describes how to connect ot path segments.
///
/// ```text
/// path-connector ::== ".." | "--"
/// ```
///
/// The connector `..` connects the paths smoothly, while `--` connects them
/// in a straight line.
#[derive(Clone, Copy, Debug)]
pub enum PathConnector {
    Straight,
    Smooth,
}

impl PathConnector {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(tag("--"), |_| PathConnector::Straight),
            map(tag(".."), |_| PathConnector::Smooth),
        ))(input)
    }

    pub fn tension(self) -> (f64, f64) {
        let res = match self {
            PathConnector::Straight => std::f64::INFINITY,
            PathConnector::Smooth => 1.
        };
        (res, res)
    }
}


//------------ Quoted --------------------------------------------------------

/// A quoted string.
///
/// This is a string in double quotes with all Rust escape sequences.
#[derive(Clone, Debug)]
pub struct Quoted {
    /// The unescaped content of the quoted string.
    pub content: String,

    /// The position of the quoted string in the input.
    pub pos: Pos
}

impl Quoted {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = tag_char('"')(input)?;
        let (input, content) = fold_many0(
            alt((
                // any character that is not a quote or a backslash stays.
                none_of("\\\""),

                // \x00..\x7F -- ASCII escape
                preceded(
                    tag("\\x"),
                    map(
                        tuple((
                            one_of("01234567"),
                            one_of("0123456789abcdefABCDEF")
                        )),
                        |(a, b)| {
                            char::try_from(
                                a.to_digit(16).unwrap() << 4
                                | b.to_digit(16).unwrap()
                            ).unwrap()
                        }
                    )
                ),

                // \u{..} -- Unicode escape. Up to 6 hex digits.
                terminated(
                    preceded(
                        tag("\\u{"),
                        map(
                            take_while_m_n(
                                1, 6, |c: char| c.is_ascii_hexdigit()
                            ),
                            |span: Span| {
                                char::try_from(
                                    u32::from_str_radix(
                                        *span.fragment(),
                                        16
                                    ).unwrap()
                                ).unwrap()
                            }
                        )
                    ),
                    tag("}")
                ),

                // \n, \r, \t, \", \\
                map(tag("\\n"), |_| '\n'),
                map(tag("\\r"), |_| '\r'),
                map(tag("\\t"), |_| '\t'),
                map(tag("\\\\"), |_| '\\'),
                map(tag("\\\""), |_| '\"'),
            )),
            String::new(),
            |mut acc: String, ch| { acc.push(ch); acc }
        )(input)?;
        let (input, _) = tag_char('"')(input)?;
        Ok((input, Quoted { content, pos }))
    }
}

impl AsRef<str> for Quoted {
    fn as_ref(&self) -> &str {
        self.content.as_ref()
    }
}


//------------ UnitNumber ----------------------------------------------------

/// A number with a unit.
///
/// ```text
/// unit-number ::= number unit:identifier
/// ```
///
/// This is a number followed without any white space by an identifier.
#[derive(Clone, Debug)]
pub struct UnitNumber {
    /// The numerical value of the unit number.
    pub number: Number,

    /// The identifier of the unit.
    pub unit: Identifier,

    /// The start of the unit number in the input.
    pub pos: Pos,
}

impl UnitNumber {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, number) = Number::parse(input)?;
        let (input, unit) = Identifier::parse(input)?;
        Ok((input, UnitNumber { number, unit, pos }))
    }
}


//============ Separators ====================================================
//
// This is everything that doesn’t end up as actual output.

/// Parses something preceded by mandatory white space.
fn ws<'a, O, F>(
    parse: F, 
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, O>
where F: Fn(Span<'a>) -> IResult<Span, O> {
    preceded(skip_ws, parse)
}

/// Parses something preceded by optional white space.
fn opt_ws<'a, O, F>(
    parse: F, 
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, O>
where F: Fn(Span<'a>) -> IResult<Span, O> {
    preceded(skip_opt_ws, parse)
}

/// Mandatory white space.
///
/// White space is all actual white space characters plus comments.
fn skip_ws(input: Span) -> IResult<Span, ()> {
    fold_many1(alt((map(multispace1, |_| ()), comment)), (), |_, _| ())(input)
}

/// Optional white space.
///
/// White space is all actual white space characters plus comments.
fn skip_opt_ws(input: Span) -> IResult<Span, ()> {
    fold_many0(alt((map(multispace1, |_| ()), comment)), (), |_, _| ())(input)
}

/// Comments start with a hash and run to the end of a line.
fn comment(input: Span) -> IResult<Span, ()> {
    let (input, _) = tag_char('#')(input)?;
    let (input, _) = take_until("\n")(input)?;
    map(tag_char('\n'), |_| ())(input)
}


//------------ Pos -----------------------------------------------------------

/// The position of an item within input.
#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub offset: usize,
    pub line: u32,
    pub col: usize,
}

impl Pos {
    fn capture(span: &Span) -> Self {
        Pos {
            offset: span.location_offset(),
            line: span.location_line(),
            col: span.get_utf8_column(),
        }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}


//============ Error ====================================================

#[derive(Clone, Debug)]
pub struct Error {
    pos: Pos,
    kind: ErrorKind,
}

impl<'a> From<(Span<'a>, ErrorKind)> for Error {
    fn from((span, kind): (Span, ErrorKind)) -> Self {
        Error {
            pos: Pos::capture(&span),
            kind
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.pos, self.kind.description())
    }
}




//============ Tests =========================================================

#[cfg(test)]
mod test {
    use super::*;

    fn remains<'a, T>(res: IResult<Span<'a>, T>) -> &'a str {
        *res.unwrap().0.fragment()
    }

    fn s(s: &str) -> Span {
        Span::new(s)
    }

    #[test]
    fn test_identifier() {
        fn parse(input: &str) -> Identifier {
            let (input, id) = Identifier::parse(s(input)).unwrap();
            assert!(input.fragment().is_empty());
            id
        }

        assert_eq!(parse("foo").as_ref(), "foo");
        assert_eq!(parse("fü1").as_ref(), "fü1");
        assert_eq!(parse("foo.bar").as_ref(), "foo.bar");
        assert_eq!(parse("_foo.bar").as_ref(), "_foo.bar");
        assert!(Identifier::parse(s("1foo")).is_err());
        assert!(Identifier::parse(s(".foo")).is_err());

        assert_eq!(*Identifier::parse(s("foo#")).unwrap().0.fragment(), "#");
        assert_eq!(*Identifier::parse(s("foo ")).unwrap().0.fragment(), " ");
    }

    #[test]
    fn test_quoted() {
        fn check(input: &str, result: &str, remain: &str) {
            let (rem, res) = Quoted::parse(s(input)).unwrap();
            assert!(result == res.as_ref());
            assert!((*rem.fragment()) == remain);
        }

        check("\"foo\"bar", "foo", "bar");
        check("\"fo\\\"o\"", "fo\"o", "");
        check("\"fo\\\\o\"", "fo\\o", "");
        check("\"fo\\x20o\"", "fo o", "");
        check("\"fo\\u{12ffe}o\"", "fo\u{12ffe}o", "");
        check("\"fo\\u{1}o\"", "fo\u{1}o", "");
        
        assert!(Quoted::parse(s("\"fo\\x90o\"")).is_err());
    }

    #[test]
    fn test_ws() {
        assert!(ws(tag("foo"))(s("foo")).is_err());
        assert_eq!(remains(ws(tag("foo"))(s("  foo"))), "");
        assert_eq!(remains(ws(tag("foo"))(s("# bla\nfoo"))), "");
        assert_eq!(remains(ws(tag("foo"))(s("# bla\n  foo"))), "");
        assert_eq!(remains(ws(tag("foo"))(s("  # bla\n  foo"))), "");
    }

    #[test]
    fn test_opt_ws() {
        assert_eq!(remains(opt_ws(tag("foo"))(s("foo"))), "");
        assert_eq!(remains(opt_ws(tag("foo"))(s("  foo"))), "");
        assert_eq!(remains(opt_ws(tag("foo"))(s("# bla\nfoo"))), "");
        assert_eq!(remains(opt_ws(tag("foo"))(s("# bla\n  foo"))), "");
        assert_eq!(remains(opt_ws(tag("foo"))(s("  # bla\n  foo"))), "");
    }

    #[test]
    fn test_skip_ws() {
        assert!(skip_ws(s("foo")).is_err());
        assert_eq!(remains(skip_ws(s(" \n\t foo"))), "foo");
    }

    #[test]
    fn test_skip_opt_ws() {
        assert_eq!(remains(skip_opt_ws(s("foo"))), "foo");
        assert_eq!(remains(skip_opt_ws(s(" \n\t foo"))), "foo");
    }

    #[test]
    fn test_comment() {
        assert_eq!(remains(comment(s("# foo\nbar"))), "bar");
    }
}

