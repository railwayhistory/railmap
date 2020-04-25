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
use nom::multi::{fold_many0, fold_many1, many0, many1, separated_list};
use nom::number::complete::recognize_float;
use nom::sequence::{preceded, terminated, tuple};

type Span<'a> = nom_locate::LocatedSpan<&'a str>;


//============ Statements ====================================================

//------------ StatementList -------------------------------------------------

/// A list of statments.
///
/// This comes in two flavours, a statement list or a statement block.
///
/// ```text
/// statement-list   ::=  *(statement)
///
/// statement-block  ::=  ( "{" statement-list "}" ) | statement
/// ```
///
/// A statement list is the root of the syntax tree. That is, a map file
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
        all_consuming(Self::parse_list)(Span::new(input))
        .map(|(_, res)| res)
        .map_err(|err| {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => err.into(),
                nom::Err::Incomplete(_) => unreachable!(),
            }
        })
    }

    fn parse_list(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, statements) = terminated(
            many0(Statement::parse),
            skip_opt_ws
        )(input)?;
        Ok((input, StatementList { statements, pos }))
    }

    fn parse_block(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        alt((
            Self::parse_curly,
            map(Statement::parse, move |stm| {
                StatementList { statements: vec![stm], pos }
            })
        ))(input)
    }

    fn parse_curly(input: Span) -> IResult<Span, Self> {
        let (input, _) = opt_ws(tag_char('{'))(input)?;
        let (input, block) = opt_ws(StatementList::parse_list)(input)?;
        let (input, _) = opt_ws(tag_char('}'))(input)?;
        Ok((input, block))
    }
}


//------------ Statement -----------------------------------------------------

/// Statements are things that actually do something.
///
/// This type enumerates all currently defined statements.
///
/// ```text
/// statement  ::=  (let | no-op | procedure | with)
/// ```
#[derive(Clone, Debug)]
pub enum Statement {
    Let(Let),
    NoOp(NoOp),
    Procedure(Procedure),
    With(With),
}

impl Statement {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(Let::parse, Statement::Let),
            map(NoOp::parse, Statement::NoOp),
            map(Procedure::parse, Statement::Procedure),
            map(With::parse, Statement::With),
        ))(input)
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


//------------ Noop ----------------------------------------------------------

/// The no-op statement does nothing.
///
/// ```text
/// no-op ::= ";"
/// ```
#[derive(Clone, Debug)]
pub struct NoOp {
    /// The start of the text statement in the source.
    pub pos: Pos
}

impl NoOp {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = opt_ws(tag_char(';'))(input)?;
        Ok((input, NoOp { pos }))
    }
}


//---------- Procedure -------------------------------------------------------

/// A procedure statement renders something onto the map.
///
/// ```text
/// procedure ::= identifier "(" argument-list ")"
/// ```
///
/// The identifier references a variable in the current scope that must be a
/// (partially applied) procedure.
#[derive(Clone, Debug)]
pub struct Procedure {
    /// The name of the procedure.
    pub ident: Identifier,

    /// The arguments of the procedure.
    pub args: ArgumentList,

    /// The start of the procedure statement in the source.
    pub pos: Pos
}

impl Procedure {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, ident) = opt_ws(Identifier::parse)(input)?;
        let (input, _) = opt_ws(tag_char('('))(input)?;
        let (input, args) = opt_ws(ArgumentList::parse_opt)(input)?;
        let (input, _) = opt_ws(tag_char(')'))(input)?;
        Ok((input, Procedure { ident, args, pos }))
    }
}


//----------- With -----------------------------------------------------------

/// The with statement executes a block with updated render parameters.
///
/// ```text
/// with ::= "with" params:assignment-list statement-block
/// ```
///
/// When the with statement is evaluated, a new scope is created as a clone
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
        let (input, block) = opt_ws(StatementList::parse_block)(input)?;
        Ok((input, With { params, block, pos }))
    }
}


//============ Assignments and Arguments =====================================

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
/// argument-list ::= argument  *("," argument)
/// ```
#[derive(Clone, Debug)]
pub struct ArgumentList {
    /// The list of arguments.
    pub arguments: Vec<Argument>,

    /// The start of the argument list in the source
    pub pos: Pos,
}

impl ArgumentList {
    fn parse_opt(input: Span) -> IResult<Span, Self> {
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


//============ Expressions ===================================================


//------------ Expression ----------------------------------------------------

/// An expression.
///
/// Expressions can be evaluated to a value of a certain type.
///
/// ```text
/// expression ::= fragment *( connector fragment )
/// ```
///
/// The expression consists of exactly one fragment, it is a _simple
/// expression,_ otherwise it is a _connected expression._
#[derive(Clone, Debug)]
pub struct Expression {
    /// The first fragment.
    pub first: Fragment,

    /// The, possibly empty, sequence of connected fragments.
    pub connected: Vec<(Connector, Fragment)>,

    /// The position of the start of the expression in the source.
    pub pos: Pos
}

impl Expression {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, first) = Fragment::parse(input)?;
        let (input, connected) = many0(
            tuple((
                opt_ws(Connector::parse),
                opt_ws(Fragment::parse),
            ))
        )(input)?;
        Ok((input, Expression { first, connected, pos }))
    }
}


//------------ Fragment ------------------------------------------------------

/// A fragment is part of an expression.
///
/// ```text
/// fragment  ::=  complex | list | vector | atom
/// ```
#[derive(Clone, Debug)]
pub enum Fragment {
    Complex(Complex),
    List(List),
    Vector(Vector),
    Atom(Atom),
}

impl Fragment {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(Complex::parse, Fragment::Complex),
            map(List::parse, Fragment::List),
            map(Vector::parse, Fragment::Vector),
            map(Atom::parse, Fragment::Atom),
        ))(input)
    }

    pub fn pos(&self) -> Pos {
        match *self {
            Fragment::Complex(ref frag) => frag.pos,
            Fragment::List(ref frag) => frag.pos,
            Fragment::Vector(ref frag) => frag.pos,
            Fragment::Atom(ref frag) => frag.pos(),
        }
    }
}


//------------ Complex -------------------------------------------------------

/// A complex expression is either an external or a section atop an external.
///
/// ```text
/// complex  ::=  extern [ section ]
/// ```
#[derive(Clone, Debug)]
pub struct Complex {
    /// The external expression.
    pub external: External,

    /// The optional section applied to the external.
    pub section: Option<Section>,

    /// The start of the complex expression in the source.
    pub pos: Pos,
}

impl Complex {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, external) = External::parse(input)?;
        let (input, section) = opt(opt_ws(Section::parse))(input)?;
        Ok((input, Complex { external, section, pos }))
    }
}


//------------ External ------------------------------------------------------

/// An external expression is a reference to a value computed externally.
///
/// ```text
/// external  ::=  identifier [ "(" argument-list ")" ]
/// ```
///
#[derive(Clone, Debug)]
pub struct External {
    /// The identifier part of the external expression.
    pub ident: Identifier,

    /// The optional arguments of a function expression.
    ///
    /// If this is present, the external expression is a function expression.
    /// It may still be empty.
    pub args: Option<ArgumentList>,

    /// The start of the function expression in the source.
    pub pos: Pos,
}

impl External {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, ident) = Identifier::parse(input)?;
        let (input, args) = opt(
            terminated(
                preceded(
                    opt_ws(tag_char('(')),
                    opt_ws(ArgumentList::parse_opt)
                ),
                opt_ws(tag_char(')'))
            )
        )(input)?;
        Ok((input, External { ident, args, pos }))
    }
}


//------------ Section -------------------------------------------------------

/// A section expression describes how to derive a section of a path.
///
/// ```text
/// section  ::=  "[" location [ "," location ] "]" *offset
/// ```
#[derive(Clone, Debug)]
pub struct Section {
    /// The start location of the section.
    pub start: Location,

    /// The optional end location of the section.
    ///
    /// If this is missing, the section describes a single point.
    pub end: Option<Location>,

    /// The sequence of offsets.
    ///
    /// This may be empty.
    pub offset: Vec<Offset>,

    /// The start of the section in the source.
    pub pos: Pos
}

impl Section {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = tag_char('[')(input)?;
        let (input, start) = opt_ws(Location::parse)(input)?;
        let (input, end) = opt(
            preceded(opt_ws(tag_char(',')), opt_ws(Location::parse))
        )(input)?;
        let (input, _) = opt_ws(tag_char(']'))(input)?;
        let (input, offset) = many0(
            opt_ws(Offset::parse)
        )(input)?;
        Ok((input, Section { start, end, offset, pos }))
    }
}


//------------ Location ------------------------------------------------------

/// A location on a path.
///
/// ```text
/// location  ::=  identifier *distance
/// ```
#[derive(Clone, Debug)]
pub struct Location {
    /// The name of a node.
    pub node: Symbol,

    /// The sequence of distances from the point.
    ///
    /// May be empty.
    pub distance: Vec<Distance>,

    /// The start of the location in the source.
    pub pos: Pos
}

impl Location {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, node) = Symbol::parse(input)?;
        let (input, distance) = many0(opt_ws(Distance::parse))(input)?;
        Ok((input, Location { node, distance, pos }))
    }
}


//------------ Distance ------------------------------------------------------

/// A distance expression is a positive or negative distance
///
/// ```text
/// distance ::= add-sub distance-val
/// ```
#[derive(Clone, Debug)]
pub struct Distance {
    /// The operator for the distance.
    pub op: AddSub,

    /// The distance value.
    pub value: UnitNumber,

    /// The position of the distance expression in the input.
    pub pos: Pos,
}

impl Distance {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, op) = AddSub::parse(input)?;
        let (input, value) = opt_ws(UnitNumber::parse)(input)?;
        Ok((input, Distance { op, value, pos }))
    }
}


//------------ Offset --------------------------------------------------------

/// An offset for a section.
///
/// ```text
/// offset  ::=  sideways | shift | angle
/// ```
#[derive(Clone, Debug)]
pub enum Offset {
    Sideways(Sideways),
    Shift(Shift),
    Angle(Angle),
}

impl Offset {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(Sideways::parse, Offset::Sideways),
            map(Shift::parse, Offset::Shift),
            map(Angle::parse, Offset::Angle),
        ))(input)
    }
}


//------------ Sideways ------------------------------------------------------

/// Describes a offset of a path sideways from its original course.
///
/// ```text
/// sideways  ::=  direction unit-number
/// ```
#[derive(Clone, Debug)]
pub struct Sideways {
    /// The direction of the sideways movement.
    pub direction: Direction,

    /// The value of the movement.
    pub value: UnitNumber,

    /// The start of the expression in the source.
    pub pos: Pos,
}

impl Sideways {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, direction) = Direction::parse(input)?;
        let (input, value) = opt_ws(UnitNumber::parse)(input)?;
        Ok((input, Sideways { direction, value, pos }))
    }
}


//------------ Shift ---------------------------------------------------------

/// Describes a translation along a given vector.
///
/// ```text
/// shift  ::=  add-sub vector
/// ```
#[derive(Clone, Debug)]
pub struct Shift {
    /// The operation of applying the shift
    pub op: AddSub,

    /// The value of the shift.
    pub value: Vector,

    /// The start of the expression in the source.
    pub pos: Pos,
}

impl Shift {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, op) = AddSub::parse(input)?;
        let (input, value) = opt_ws(Vector::parse)(input)?;
        Ok((input, Shift { op, value, pos }))
    }
}


//------------ Angle ---------------------------------------------------------

/// An angle expression describes an angle.
///
/// ```text
/// angle  ::=  "@" number
/// ```
#[derive(Clone, Debug)]
pub struct Angle {
    /// The angle’s value in degrees.
    pub value: Number,

    /// The start of the angle expression in the source.
    pub pos: Pos
}

impl Angle {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = tag_char('@')(input)?;
        let (input, value) = opt_ws(Number::parse)(input)?;
        Ok((input, Angle { value, pos }))
    }
}


//------------ List ----------------------------------------------------------

/// A list expression is a possibly empty list of expressions.
///
/// ```text
/// list  ::=  "[" "]" | "[" expression *( "," expression ) "]"
/// ```
#[derive(Clone, Debug)]
pub struct List {
    /// The content of the list.
    pub content: Vec<Expression>,

    /// The start of the list expression in the source.
    pub pos: Pos
}

impl List {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, content) = terminated(
            preceded(
                tag_char('['),
                many0(opt_ws(Expression::parse))
            ),
            opt_ws(tag_char(']'))
        )(input)?;
        Ok((input, List { content, pos }))
    }
}


//------------ Vector --------------------------------------------------------

/// A vector expression provides a two-dimensional vector.
///
/// ```text
/// vector  ::=  "(" unit-number "," unit-number ")"
/// ```
#[derive(Clone, Debug)]
pub struct Vector {
    /// The x coordinate of the vector.
    pub x: UnitNumber,

    /// The y coordinate of the vector.
    pub y: UnitNumber,

    /// The start of the vector expression in the source.
    pub pos: Pos
}

impl Vector {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = tag_char('(')(input)?;
        let (input, x) = opt_ws(UnitNumber::parse)(input)?;
        let (input, _) = opt_ws(tag_char(','))(input)?;
        let (input, y) = opt_ws(UnitNumber::parse)(input)?;
        let (input, _) = opt_ws(tag_char(')'))(input)?;
        Ok((input, Vector { x, y, pos }))
    }
}


//------------ Atom ----------------------------------------------------------

/// An atom is a basic expression.
///
/// ```text
/// atom  ::=  number | symbol-set | text | unit-number
/// ```
#[derive(Clone, Debug)]
pub enum Atom {
    Number(Number),
    SymbolSet(SymbolSet),
    Text(Text),
    UnitNumber(UnitNumber),
}

impl Atom {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(Number::parse, Atom::Number),
            map(SymbolSet::parse, Atom::SymbolSet),
            map(Text::parse, Atom::Text),
            map(UnitNumber::parse, Atom::UnitNumber),
        ))(input)
    }

    pub fn pos(&self) -> Pos {
        match *self {
            Atom::Number(ref atom) => atom.pos,
            Atom::SymbolSet(ref atom) => atom.pos,
            Atom::Text(ref atom) => atom.pos,
            Atom::UnitNumber(ref atom) => atom.pos,
        }
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


//------------ Symbol --------------------------------------------------------

/// A symbolic name for something.
///
/// ```text
/// symbol  ::=  <a colon directly followed by an indentifer>
/// ```
#[derive(Clone, Debug)]
pub struct Symbol {
    /// The identifier portion of the symbol.
    pub ident: Identifier,

    /// The start of the symbol in the source
    pub pos: Pos,
}

impl Symbol {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = tag_char(':')(input)?;
        let (input, ident) = Identifier::parse(input)?;
        Ok((input, Symbol { ident, pos }))
    }
}

impl From<Symbol> for String {
    fn from(sym: Symbol) -> String {
        sym.ident.ident
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.ident.as_ref()
    }
}

impl borrow::Borrow<str> for Symbol {
    fn borrow(&self) -> &str {
        self.ident.as_ref()
    }
}

impl<T: AsRef<str>> PartialEq<T> for Symbol {
    fn eq(&self, other: &T) -> bool {
        self.ident == other.as_ref()
    }
}

impl Eq for Symbol { }

impl hash::Hash for Symbol {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ident.fmt(f)
    }
}


//------------ SymbolSet -----------------------------------------------------

/// A sequence of symbols.
///
/// ```text
/// symbol-set  ::=  symbol *(symbol)
/// ```
#[derive(Clone, Debug)]
pub struct SymbolSet {
    /// The set of symbols.
    pub symbols: Vec<Symbol>,

    /// The start of the set in the source.
    pub pos: Pos
}

impl SymbolSet {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, symbols) = many1(opt_ws(Symbol::parse))(input)?;
        Ok((input, SymbolSet { symbols, pos }))
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
    /// allocate in the likely case that there aren’t any. 
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


//------------ UnitNumber ----------------------------------------------------

/// A number with a unit.
///
/// ```text
/// unit-number ::= <number follwed directly by unit identifier>
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


//============ Operators =====================================================
//
// Operators are fixed sequences that connect other expressions in a sepecific
// way.

//------------ AddSub --------------------------------------------------------

/// Adding or subtracting two expressions.
///
/// ```text
/// add-sub ::= "+" | "-"
/// ```
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


//------------ Connector -----------------------------------------------------

/// A connector describes how to connect two segments.
///
/// ```text
/// path-connector ::== ".." | "--"
/// ```
///
/// The connector `..` connects the paths smoothly, while `--` connects them
/// in a straight line.
#[derive(Clone, Copy, Debug)]
pub enum Connector {
    Straight,
    Smooth,
}

impl Connector {
    fn parse(input: Span) -> IResult<Span, Self> {
        alt((
            map(tag("--"), |_| Connector::Straight),
            map(tag(".."), |_| Connector::Smooth),
        ))(input)
    }

    pub fn tension(self) -> (f64, f64) {
        let res = match self {
            Connector::Straight => std::f64::INFINITY,
            Connector::Smooth => 1.
        };
        (res, res)
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

