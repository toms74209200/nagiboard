use std::fmt;
use std::iter;
use std::ops::ControlFlow;

use crate::board::{Board, EdgeRejected, Line};
use crate::note_id::NoteId;
use crate::note_type::NoteType;

const HEADER: &str = "# eventstorming v1";

#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub board: Board,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: usize,
    pub kind: DiagnosticKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    UnknownNoteType(String),
    DuplicateNoteId(String),
    UnparsableLine(String),
    UndefinedReference { from: String, to: String },
    DuplicateEdge { from: String, to: String },
    CoercedToDashed { from: String, to: String },
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ln = self.line;
        match &self.kind {
            DiagnosticKind::UnknownNoteType(t) => write!(f, "{ln}行目: 未知の付箋タイプ \"{t}\""),
            DiagnosticKind::DuplicateNoteId(id) => {
                write!(f, "{ln}行目: ID \"{id}\" が重複しています")
            }
            DiagnosticKind::UnparsableLine(line) => write!(f, "{ln}行目: 解釈できません → {line}"),
            DiagnosticKind::UndefinedReference { from, to } => {
                write!(f, "{ln}行目: 未定義のIDを参照しています ({from} / {to})")
            }
            DiagnosticKind::DuplicateEdge { from, to } => {
                write!(f, "{ln}行目: 接続が重複しています ({from} -> {to})")
            }
            DiagnosticKind::CoercedToDashed { from, to } => {
                write!(
                    f,
                    "{ln}行目: ルール外の接続のため点線にしました ({from} -> {to})"
                )
            }
        }
    }
}

pub fn serialize(board: &Board) -> String {
    let title = board
        .title()
        .trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
    let pad = board
        .notes()
        .iter()
        .map(|n| n.note_type().as_str().len())
        .fold(9, usize::max);
    let notes = board.notes().iter().map(|n| {
        format!(
            "{:<pad$} {:<4} \"{}\" @ {},{}",
            n.note_type().as_str(),
            n.id().as_str(),
            n.text()
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n"),
            (n.x() + 0.5).floor() as i64,
            (n.y() + 0.5).floor() as i64,
        )
    });
    let edges = board.edges().iter().map(|e| {
        let arrow = match e.line() {
            Line::Solid => "->",
            Line::Dashed => "..>",
        };
        format!("{} {arrow} {}", e.from(), e.to())
    });
    iter::once(HEADER.to_string())
        .chain((!title.is_empty()).then(|| format!("title {title}")))
        .chain(iter::once(String::new()))
        .chain(notes)
        .chain((!board.edges().is_empty()).then(String::new))
        .chain(edges)
        .map(|line| line + "\n")
        .collect()
}

enum Statement<'a> {
    Title(&'a str),
    Note(NoteType, NoteDecl<'a>),
    Edge(&'a str, Line, &'a str),
}

pub fn parse(text: &str) -> Parsed {
    let statements: Vec<(usize, Result<Statement, DiagnosticKind>)> = text
        .split('\n')
        .enumerate()
        .filter_map(|(idx, raw)| {
            let raw = raw.strip_suffix('\r').unwrap_or(raw);
            let end = match raw.char_indices().try_fold(
                (None, false),
                |(quote_start, escaped), (i, c)| match (quote_start, escaped, c) {
                    (Some(start), true, _) => ControlFlow::Continue((Some(start), false)),
                    (Some(start), false, '\\') => ControlFlow::Continue((Some(start), true)),
                    (Some(_), false, '"') => ControlFlow::Continue((None, false)),
                    (None, _, '"') => ControlFlow::Continue((Some(i), false)),
                    (None, _, '#') => ControlFlow::Break(i),
                    (state, escaped, _) => ControlFlow::Continue((state, escaped)),
                },
            ) {
                ControlFlow::Break(i) => i,
                ControlFlow::Continue((Some(start), true)) => start,
                ControlFlow::Continue(_) => raw.len(),
            };
            let line = raw[..end].trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
            (!line.is_empty()).then_some((idx + 1, line))
        })
        .map(|(ln, line)| {
            let statement = if let Some(title) = line
                .strip_prefix("title")
                .filter(|t| t.starts_with(|c: char| c.is_whitespace() || c == '\u{FEFF}'))
            {
                Ok(Statement::Title(title.trim_matches(|c: char| {
                    c.is_whitespace() || c == '\u{FEFF}'
                })))
            } else if let Some(decl) = parse_note_line(line) {
                match NoteType::parse(&decl.note_type.to_ascii_lowercase()) {
                    Some(note_type) => Ok(Statement::Note(note_type, decl)),
                    None => Err(DiagnosticKind::UnknownNoteType(decl.note_type.to_string())),
                }
            } else if let Some((from, requested, to)) = parse_edge_line(line) {
                Ok(Statement::Edge(from, requested, to))
            } else {
                Err(DiagnosticKind::UnparsableLine(line.to_string()))
            };
            (ln, statement)
        })
        .collect();

    let (board, diagnostics) = statements.iter().fold(
        (Board::default(), Vec::new()),
        |(mut board, mut diagnostics), (ln, statement)| {
            let kind = match statement {
                Ok(Statement::Title(title)) => {
                    board.set_title(title.to_string());
                    None
                }
                Ok(Statement::Note(note_type, decl)) => {
                    let fallback = 40.0 + board.notes().len() as f64 * 30.0;
                    let (x, y) = decl.position.unwrap_or((fallback, fallback));
                    board
                        .insert_note(decl.id.clone(), *note_type, decl.text.clone(), x, y)
                        .err()
                        .map(|_| DiagnosticKind::DuplicateNoteId(decl.id.to_string()))
                }
                Ok(Statement::Edge(..)) => None,
                Err(kind) => Some(kind.clone()),
            };
            diagnostics.extend(kind.map(|kind| Diagnostic { line: *ln, kind }));
            (board, diagnostics)
        },
    );

    let (board, diagnostics) = statements.iter().fold(
        (board, diagnostics),
        |(mut board, mut diagnostics), (ln, statement)| {
            let Ok(Statement::Edge(from, requested, to)) = statement else {
                return (board, diagnostics);
            };
            let inserted = match (NoteId::parse(from), NoteId::parse(to)) {
                (Some(from), Some(to)) => board.insert_edge(&from, &to, *requested),
                _ => Err(EdgeRejected::MissingEndpoint),
            };
            let (from, to) = (from.to_string(), to.to_string());
            let kind = match inserted {
                Ok(line) if line == *requested => None,
                Ok(_) => Some(DiagnosticKind::CoercedToDashed { from, to }),
                Err(EdgeRejected::MissingEndpoint) => {
                    Some(DiagnosticKind::UndefinedReference { from, to })
                }
                Err(EdgeRejected::Duplicate) => Some(DiagnosticKind::DuplicateEdge { from, to }),
            };
            diagnostics.extend(kind.map(|kind| Diagnostic { line: *ln, kind }));
            (board, diagnostics)
        },
    );

    Parsed { board, diagnostics }
}

struct NoteDecl<'a> {
    note_type: &'a str,
    id: NoteId,
    text: String,
    position: Option<(f64, f64)>,
}

fn parse_note_line(line: &str) -> Option<NoteDecl<'_>> {
    let (note_type, rest) = line.split_once(|c: char| c.is_whitespace() || c == '\u{FEFF}')?;
    if !note_type
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return None;
    }
    let (raw_id, rest) = rest
        .trim_start_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}')
        .split_once(|c: char| c.is_whitespace() || c == '\u{FEFF}')?;
    let id = NoteId::parse(raw_id)?;
    let body = rest
        .trim_start_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}')
        .strip_prefix('"')?;
    let (text, close) =
        match body
            .char_indices()
            .try_fold(
                (String::new(), false),
                |(mut text, escaped), (i, c)| match (escaped, c) {
                    (true, c) => {
                        text.push(if c == 'n' { '\n' } else { c });
                        ControlFlow::Continue((text, false))
                    }
                    (false, '\\') => ControlFlow::Continue((text, true)),
                    (false, '"') => ControlFlow::Break((text, i)),
                    (false, c) => {
                        text.push(c);
                        ControlFlow::Continue((text, false))
                    }
                },
            ) {
            ControlFlow::Break(parsed) => parsed,
            ControlFlow::Continue(_) => return None,
        };
    let rest = body[close + 1..].trim_start_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
    let position = match rest.strip_prefix('@') {
        None if rest.is_empty() => None,
        None => return None,
        Some(coordinates) => {
            let (x, y) = coordinates.split_once(',')?;
            Some((
                parse_number(x.trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}'))?,
                parse_number(y.trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}'))?,
            ))
        }
    };
    Some(NoteDecl {
        note_type,
        id,
        text,
        position,
    })
}

fn parse_edge_line(line: &str) -> Option<(&str, Line, &str)> {
    let gt = line.rfind('>')?;
    let head = &line[..gt];
    let (left, requested) = if let Some(left) = head.strip_suffix("..") {
        (left, Line::Dashed)
    } else {
        (head.strip_suffix('-')?, Line::Solid)
    };
    let from = left.trim_end_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
    let to = line[gt + 1..].trim_start_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
    [from, to]
        .iter()
        .all(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
        .then_some((from, requested, to))
}

fn parse_number(s: &str) -> Option<f64> {
    let unsigned = s.strip_prefix('-').unwrap_or(s);
    let (int, frac) = unsigned.split_once('.').unwrap_or((unsigned, "0"));
    let is_digits = |p: &str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit());
    (is_digits(int) && is_digits(frac))
        .then(|| s.parse().ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use random_string::CharacterType::{Lowercase, Numeric};
    use random_string::generate_random_string;
    use std::fs::File;

    #[test]
    fn when_parse_escaped_backslash_then_unescapes_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (a, b) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let dsl = format!(
            "event {} \"{a}\\\\{b}\"",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.notes(),
            [n] if n.text() == format!("{a}\\{b}")
        ));
    }

    #[test]
    fn when_parse_escaped_quote_then_unescapes_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (a, b) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let dsl = format!(
            "event {} \"{a}\\\"{b}\"",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.notes(),
            [n] if n.text() == format!("{a}\"{b}")
        ));
    }

    #[test]
    fn when_parse_escaped_n_then_becomes_newline() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (a, b) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let dsl = format!(
            "event {} \"{a}\\n{b}\"",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.notes(),
            [n] if n.text() == format!("{a}\n{b}")
        ));
    }

    #[test]
    fn when_serialize_text_with_backslash_quote_and_newline_then_escapes_them() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "event     {} \"{}\\\\{}\\\"{}\\n{}\" @ 0,0",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let text = format!("# eventstorming v1\n\n{line}\n");
        assert_eq!(serialize(&parse(&text).board), text);
    }

    #[test]
    fn when_serialize_parsed_canonical_text_then_reproduces_it() {
        let text = r#"# eventstorming v1
title 注文フロー

actor     a1   "顧客" @ 48,232
command   c1   "注文を確定する" @ 216,224
aggregate g1   "注文" @ 416,208
event     e1   "注文が確定した" @ 672,224
policy    p1   "" @ 880,120
hotspot   h1   "キャンセル期限は?" @ 672,56

a1 -> c1
c1 -> g1
g1 -> e1
e1 -> p1
e1 ..> h1
"#;
        assert_eq!(serialize(&parse(text).board), text);
    }

    #[test]
    fn when_serialize_empty_board_then_writes_only_header() {
        assert_eq!(serialize(&Board::default()), "# eventstorming v1\n\n");
    }

    #[test]
    fn when_serialize_with_half_coordinates_then_rounds_half_up() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let dsl = format!(
            "event {} \"{}\" @ 0.5,-2.5",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(serialize(&parse(&dsl).board).contains("@ 1,-2"));
    }

    #[test]
    fn when_parse_note_line_then_reads_every_field() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (id, text) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let (x, y): (f64, f64) = (
            generate_random_string(3, &[Numeric], "", &mut urandom)
                .parse()
                .unwrap(),
            generate_random_string(3, &[Numeric], "", &mut urandom)
                .parse()
                .unwrap(),
        );
        assert!(matches!(
            parse(&format!("actor {id} \"{text}\" @ {x},{y}")),
            Parsed { ref board, ref diagnostics }
                if diagnostics.is_empty()
                    && matches!(board.notes(), [n] if n.id().as_str() == id
                        && n.note_type() == NoteType::Actor
                        && n.text() == text
                        && (n.x(), n.y()) == (x, y))
        ));
    }

    #[test]
    fn when_parse_with_comment_after_declaration_then_ignores_the_comment() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (a, b) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let dsl = format!(
            "event {} \"{a} # {b}\" @ 0,0 # {}",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.notes(),
            [n] if n.text() == format!("{a} # {b}")
        ));
    }

    #[test]
    fn when_parse_without_coordinates_then_assigns_cascading_defaults() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let dsl = format!(
            "event {} \"{}\"\nevent {} \"{}\"\n",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.notes(),
            [a, b] if (a.x(), a.y(), b.x(), b.y()) == (40.0, 40.0, 70.0, 70.0)
        ));
    }

    #[test]
    fn when_parse_with_crlf_then_parses_like_lf() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (title, text) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let dsl = format!(
            "title {title}\r\nevent {} \"{text}\"\r\n",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl),
            Parsed { ref board, ref diagnostics }
                if diagnostics.is_empty() && board.title() == title && board.notes()[0].text() == text
        ));
    }

    #[test]
    fn when_parse_with_byte_order_mark_then_ignores_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = generate_random_string(8, &[Lowercase], "", &mut urandom);
        let dsl = format!(
            "\u{FEFF}event {id} \"{}\"",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.notes(),
            [n] if n.id().as_str() == id
        ));
    }

    #[test]
    fn when_parse_with_uppercase_type_then_accepts_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let dsl = format!(
            "EVENT {} \"{}\"",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.notes(),
            [n] if n.note_type() == NoteType::Event
        ));
    }

    #[test]
    fn when_parse_with_decimal_and_negative_coordinates_then_keeps_them() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let dsl = format!(
            "event {} \"{}\" @ -1.5 , 2.25",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.notes(),
            [n] if (n.x(), n.y()) == (-1.5, 2.25)
        ));
    }

    #[test]
    fn when_parse_with_hyphenated_ids_then_connects_them() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (from, to) = (
            format!(
                "{}-{}",
                generate_random_string(8, &[Lowercase], "", &mut urandom),
                generate_random_string(8, &[Lowercase], "", &mut urandom)
            ),
            format!(
                "{}-{}",
                generate_random_string(8, &[Lowercase], "", &mut urandom),
                generate_random_string(8, &[Lowercase], "", &mut urandom)
            ),
        );
        let dsl = format!(
            "actor {from} \"{}\"\ncommand {to} \"{}\"\n{from}->{to}\n",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).board.edges(),
            [e] if e.from().as_str() == from && e.to().as_str() == to && e.line() == Line::Solid
        ));
    }

    #[test]
    fn when_parse_with_unknown_type_then_reports_line_number() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let note_type = generate_random_string(8, &[Lowercase], "", &mut urandom);
        let dsl = format!(
            "{note_type} {} \"{}\"",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 未知の付箋タイプ \"{note_type}\"")
        ));
    }

    #[test]
    fn when_parse_with_duplicated_id_then_keeps_first_and_reports_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (id, text) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let parsed = parse(&format!(
            "event {id} \"{text}\"\nevent {id} \"{}\"",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        ));
        assert!(matches!(
            (parsed.board.notes(), parsed.diagnostics.as_slice()),
            ([n], [d]) if n.text() == text
                && d.to_string() == format!("2行目: ID \"{id}\" が重複しています")
        ));
    }

    #[test]
    fn when_parse_with_undefined_edge_reference_then_reports_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (id, missing) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let dsl = format!(
            "event {id} \"{}\"\n{id} -> {missing}",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).diagnostics.as_slice(),
            [d] if d.to_string() == format!("2行目: 未定義のIDを参照しています ({id} / {missing})")
        ));
    }

    #[test]
    fn when_parse_with_garbage_line_then_reports_it() {
        assert!(matches!(
            parse("?!").diagnostics.as_slice(),
            [d] if d.to_string() == "1行目: 解釈できません → ?!"
        ));
    }

    #[test]
    fn when_parse_with_unterminated_quote_then_reports_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "event {} \"{}",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&line).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → {line}")
        ));
    }

    #[test]
    fn when_parse_with_duplicated_edge_then_keeps_first_and_reports_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (from, to) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let parsed = parse(&format!(
            "actor {from} \"{}\"\ncommand {to} \"{}\"\n{from} -> {to}\n{from} ..> {to}\n",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        ));
        assert!(matches!(
            (parsed.board.edges(), parsed.diagnostics.as_slice()),
            ([e], [d]) if e.line() == Line::Solid
                && d.to_string() == format!("4行目: 接続が重複しています ({from} -> {to})")
        ));
    }

    #[test]
    fn when_parse_with_out_of_rule_solid_edge_then_coerces_to_dashed_and_reports_it() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (from, to) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let parsed = parse(&format!(
            "command {from} \"{}\"\nactor {to} \"{}\"\n{from} -> {to}\n",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        ));
        assert!(matches!(
            (parsed.board.edges(), parsed.diagnostics.as_slice()),
            ([e], [d]) if e.line() == Line::Dashed
                && d.to_string() == format!("3行目: ルール外の接続のため点線にしました ({from} -> {to})")
        ));
    }

    #[test]
    fn when_parse_with_trailing_backslash_in_unterminated_quote_then_cuts_line_at_quote() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = generate_random_string(8, &[Lowercase], "", &mut urandom);
        let dsl = format!(
            "event {id} \"{}\\",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → event {id}")
        ));
    }

    #[test]
    fn when_parse_with_edge_reference_starting_with_digit_then_reports_undefined_reference() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let id = generate_random_string(8, &[Lowercase], "", &mut urandom);
        let reference = generate_random_string(1, &[Numeric], "", &mut urandom)
            + &generate_random_string(8, &[Lowercase], "", &mut urandom);
        let dsl = format!(
            "event {id} \"{}\"\n{id} -> {reference}",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl).diagnostics.as_slice(),
            [d] if d.to_string() == format!("2行目: 未定義のIDを参照しています ({id} / {reference})")
        ));
    }

    #[test]
    fn when_parse_with_symbol_in_type_then_reports_unparsable_line() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "{}!{} {} \"{}\"",
            generate_random_string(4, &[Lowercase], "", &mut urandom),
            generate_random_string(4, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&line).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → {line}")
        ));
    }

    #[test]
    fn when_parse_with_byte_order_mark_between_tokens_then_treats_it_as_space() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (id, title) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let dsl = format!(
            "title\u{FEFF}{title}\nevent\u{FEFF}{id}\u{FEFF}\"{}\"\u{FEFF}@\u{FEFF}1\u{FEFF},\u{FEFF}2",
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&dsl),
            Parsed { ref board, ref diagnostics }
                if diagnostics.is_empty()
                    && board.title() == title
                    && matches!(board.notes(), [n] if n.id().as_str() == id)
        ));
    }

    #[test]
    fn when_parse_without_quotes_around_text_then_reports_unparsable_line() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "event {} {}",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&line).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → {line}")
        ));
    }

    #[test]
    fn when_parse_with_coordinates_without_comma_then_reports_unparsable_line() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "event {} \"{}\" @ 1 2",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&line).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → {line}")
        ));
    }

    #[test]
    fn when_parse_with_exponent_coordinate_then_reports_unparsable_line() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "event {} \"{}\" @ 1,1e3",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&line).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → {line}")
        ));
    }

    #[test]
    fn when_parse_with_sign_only_coordinate_then_reports_unparsable_line() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "event {} \"{}\" @ -,1",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&line).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → {line}")
        ));
    }

    #[test]
    fn when_parse_with_bare_greater_than_then_reports_unparsable_line() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "{} > {}",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&line).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → {line}")
        ));
    }

    #[test]
    fn when_parse_with_trailing_words_after_text_then_reports_unparsable_line() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let line = format!(
            "event {} \"{}\" {}",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        );
        assert!(matches!(
            parse(&line).diagnostics.as_slice(),
            [d] if d.to_string() == format!("1行目: 解釈できません → {line}")
        ));
    }

    #[test]
    fn when_parse_with_explicit_dashed_edge_then_keeps_it_without_diagnostics() {
        let mut urandom = File::open("/dev/urandom").unwrap();
        let (from, to) = (
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom),
        );
        let parsed = parse(&format!(
            "event {from} \"{}\"\nhotspot {to} \"{}\"\n{from} ..> {to}\n",
            generate_random_string(8, &[Lowercase], "", &mut urandom),
            generate_random_string(8, &[Lowercase], "", &mut urandom)
        ));
        assert!(matches!(
            (parsed.board.edges(), parsed.diagnostics.as_slice()),
            ([e], []) if e.line() == Line::Dashed
        ));
    }
}
