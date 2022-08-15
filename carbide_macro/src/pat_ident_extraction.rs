use std::collections::HashSet;
use proc_macro2::{Ident, Span};
use syn::{Arm, Expr, parse_quote, Pat};

pub fn extract_idents_from_pattern(pat: Pat) -> Vec<Ident> {
    match pat {
        // Getting ident from an ident pattern is just taking the ident.
        Pat::Ident(i) => {
            vec![i.ident]
        }
        // Getting idents from a box pattern is just extracting from the inner pat
        Pat::Box(i) => {
            extract_idents_from_pattern(*i.pat)
        }
        // Fixme: We currently dont look after idents in literals. We should because they contain expr
        Pat::Lit(_) => {
            vec![]
        }
        // Fixme: We currently dont look after idents in macros. They contain a tokenstream but it will be hard to parse.
        Pat::Macro(_) => {
            vec![]
        }
        // In the or pattern, we need to ensure the idents from each case are the same.
        // We do not do any type checking, but only ensures the names match
        Pat::Or(i) => {
            if i.cases.len() == 0 {
                return vec![]
            }

            let mut set = HashSet::new();
            set.extend(extract_idents_from_pattern(i.cases[0].clone()));

            for case in i.cases.iter().skip(1) {
                let idents_from_case = HashSet::from_iter(extract_idents_from_pattern(case.clone()));
                if set != idents_from_case {
                    panic!("We have an or case where the binding idents names from one does not match that from the other.")
                }
            }

            Vec::from_iter(set)
        }
        // Even when the path contains idents, they are for structs and such and not used for bindings?
        Pat::Path(_) => {
            vec![]
        }
        // Can you even match using a range pat with an ident? I guess most would use literals?
        Pat::Range(_) => {
            vec![]
        }
        // From the reference, we can just extract from the inner pattern.
        Pat::Reference(i) => {
            extract_idents_from_pattern(*i.pat)
        }
        // From the rest, we have no idents.
        Pat::Rest(_) => {
            vec![]
        }
        // From the slice we check if all the idents from the different patterns are unique.
        // Otherwise we fail.
        Pat::Slice(i) => {
            let mut res = vec![];
            for elem in i.elems {
                let elem_idents = extract_idents_from_pattern(elem);
                if res.iter().any(|i| {
                    elem_idents.contains(i)
                }) {
                    panic!("Multiple slices contain the same ident names.")
                }

                res.extend(elem_idents);
            }

            res
        }
        // From the struct we check if all the idents from the different patterns are unique.
        // Otherwise we fail.
        Pat::Struct(i) => {
            let mut res = vec![];

            for field in i.fields.iter().map(|a| a.pat.clone()) {
                let field_idents = extract_idents_from_pattern(*field);
                if res.iter().any(|i| {
                    field_idents.contains(i)
                }) {
                    panic!("Multiple slices contain the same ident names.")
                }

                res.extend(field_idents);
            }

            res
        }
        // From the tuple we check if all the idents from the different patterns are unique.
        // Otherwise we fail.
        Pat::Tuple(i) => {
            let mut res = vec![];
            for elem in i.elems {
                let elem_idents = extract_idents_from_pattern(elem);
                if res.iter().any(|i| {
                    elem_idents.contains(i)
                }) {
                    panic!("Multiple slices contain the same ident names.")
                }

                res.extend(elem_idents);
            }

            res
        }
        // From the tuple struct we check if all the idents from the different patterns are unique.
        // Otherwise we fail.
        Pat::TupleStruct(i) => {
            let mut res = vec![];

            for field in i.pat.elems {
                let field_idents = extract_idents_from_pattern(field);
                if res.iter().any(|i| {
                    field_idents.contains(i)
                }) {
                    panic!("Multiple slices contain the same ident names.")
                }

                res.extend(field_idents);
            }

            res
        }
        Pat::Type(i) => {
            extract_idents_from_pattern(*i.pat)
        }
        // No idents in verbatim. Hopefully we dont hit this ever?
        Pat::Verbatim(i) => {
            panic!("{}", i.to_string());
        }
        // No ident exist in wildcard patterns
        Pat::Wild(_) => {
            vec![]
        }

        #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
        _ => { panic!("Pat is a new token not yet matched with") }
    }
}


mod tests {
    use proc_macro2::{Ident, Span};
    use syn::{parse_quote, Pat};
    use crate::pat_ident_extraction::extract_idents_from_pattern;

    #[test]
    fn extract_from_lit() {
        // Arrange
        let pat: Pat = parse_quote!(
            1
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_ident() {
        // Arrange
        let pat: Pat = parse_quote!(
            x
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site())
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_boxed_ident() {
        // Arrange
        let pat: Pat = parse_quote!(
            box x
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site())
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_boxed_lit() {
        // Arrange
        let pat: Pat = parse_quote!(
            box 1
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_boxed_boxed_ident() {
        // Arrange
        let pat: Pat = parse_quote!(
            box box x
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site())
        ];

        assert_eq!(expected, actual)
    }

    // Not sure the or is actually parsable currently :/
    /*#[test]
    fn extract_from_or_idents() {
        // Arrange
        let pat: Pat = parse_quote!(
            x or x
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site())
        ];

        assert_eq!(expected, actual)
    }*/

    #[test]
    fn extract_from_referenced_ident() {
        // Arrange
        let pat: Pat = parse_quote!(
            &x
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site())
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_slice() {
        // Arrange
        let pat: Pat = parse_quote!(
            [a, b, ref i @ .., y, z]
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
            Ident::new("b", Span::call_site()),
            Ident::new("i", Span::call_site()),
            Ident::new("y", Span::call_site()),
            Ident::new("z", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_slice_no_idents() {
        // Arrange
        let pat: Pat = parse_quote!(
            [1, 2]
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    #[should_panic]
    fn extract_from_slice_multiple_same_idents() {
        // Arrange
        let pat: Pat = parse_quote!(
            [a, a]
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat); // Should panic

        // Assert
    }

    #[test]
    fn extract_from_struct() {
        // Arrange
        let pat: Pat = parse_quote!(
            Variant { x, y, .. }
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_empty_struct() {
        // Arrange
        let pat: Pat = parse_quote!(
            Variant { .. }
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    #[should_panic]
    fn extract_from_struct_multiple_same_idents() {
        // Arrange
        let pat: Pat = parse_quote!(
            Variant { x, x, .. }
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
    }

    #[test]
    fn extract_from_tuple() {
        // Arrange
        let pat: Pat = parse_quote!(
            (a, 1)
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_tuple_no_idents() {
        // Arrange
        let pat: Pat = parse_quote!(
            (1, 2, 3)
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    #[should_panic]
    fn extract_from_tuple_multiple_same_idents() {
        // Arrange
        let pat: Pat = parse_quote!(
            (a, a)
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
    }

    #[test]
    fn extract_from_tuple_struct() {
        // Arrange
        let pat: Pat = parse_quote!(
            Variant ( x, y, .. )
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_empty_tuple_struct() {
        // Arrange
        let pat: Pat = parse_quote!(
            Variant ( .. )
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    #[should_panic]
    fn extract_from_tuple_struct_multiple_same_idents() {
        // Arrange
        let pat: Pat = parse_quote!(
            Variant ( x, x, .. )
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
    }

    /*#[test]
    fn extract_from_type() {
        // Arrange
        let pat: Pat = parse_quote!(
            x: u32
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }*/

    /*#[test]
    fn extract_from_type_no_idents() {
        // Arrange
        let pat: Pat = parse_quote!(
            1: u32
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }*/

    #[test]
    fn extract_from_wildcard() {
        // Arrange
        let pat: Pat = parse_quote!(
            _
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_complex_1() {
        // Arrange
        let pat: Pat = parse_quote!(
            Graph { editing_mode: EditingMode::Selection { selected: SelectedState::Node(x), .. }, .. }
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_complex_2() {
        // Arrange
        let pat: Pat = parse_quote!(
            Graph { editing_mode: [a, e, ref i @ .., _, t], other: (s, g), dark: Test1(c), .. }
        );
        println!("{:#?}", &pat);

        // Act
        let actual = extract_idents_from_pattern(pat);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
            Ident::new("e", Span::call_site()),
            Ident::new("i", Span::call_site()),
            Ident::new("t", Span::call_site()),
            Ident::new("s", Span::call_site()),
            Ident::new("g", Span::call_site()),
            Ident::new("c", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

}
