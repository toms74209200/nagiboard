pub fn board_from_dsl(dsl: &str) -> Result<keel::board::Board, Vec<keel::dsl::Diagnostic>> {
    let parsed = keel::dsl::parse(dsl);
    let rejected: Vec<keel::dsl::Diagnostic> = parsed
        .diagnostics
        .into_iter()
        .filter(|d| !matches!(d.kind, keel::dsl::DiagnosticKind::CoercedToDashed { .. }))
        .collect();
    if rejected.is_empty() {
        Ok(parsed.board)
    } else {
        Err(rejected)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn when_board_from_dsl_with_note_line_then_returns_board_with_the_note() {
        let id = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let text = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );

        assert!(matches!(
            super::board_from_dsl(&format!("event {id} \"{text}\" @ 0,0\n")),
            Ok(board) if matches!(
                board.notes(),
                [n] if n.id().as_str() == id && n.text() == text
            )
        ));
    }

    #[test]
    fn when_board_from_dsl_with_solid_edge_against_rules_then_returns_board_with_dashed_edge() {
        let from = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let to = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Uppercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );

        assert!(matches!(
            super::board_from_dsl(&format!(
                "event {from} \"\" @ 0,0\nactor {to} \"\" @ 0,0\n{from} -> {to}\n"
            )),
            Ok(board) if matches!(board.edges(), [e] if e.line() == keel::board::Line::Dashed)
        ));
    }

    #[test]
    fn when_board_from_dsl_with_unknown_note_type_then_returns_its_diagnostic() {
        let note_type = random_string::generate_random_string(
            12,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );

        assert_eq!(
            super::board_from_dsl(&format!("{note_type} x1 \"\" @ 0,0\n")),
            Err(vec![keel::dsl::Diagnostic {
                line: 1,
                kind: keel::dsl::DiagnosticKind::UnknownNoteType(note_type),
            }])
        );
    }

    #[test]
    fn when_board_from_dsl_with_reference_to_undefined_note_then_returns_its_diagnostic() {
        let from = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let to = random_string::generate_random_string(
            8,
            &[random_string::CharacterType::Uppercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );

        assert!(matches!(
            super::board_from_dsl(&format!("event {from} \"\" @ 0,0\n{from} -> {to}\n")),
            Err(diagnostics) if matches!(
                diagnostics.as_slice(),
                [keel::dsl::Diagnostic {
                    line: 2,
                    kind: keel::dsl::DiagnosticKind::UndefinedReference { .. },
                }]
            )
        ));
    }
}
