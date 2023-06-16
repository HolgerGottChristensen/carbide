use std::collections::HashSet;
use proc_macro2::Ident;
use syn::Expr;

/// Return the idents that are unbound
pub fn extract_idents_from_expression(expr: Expr) -> Vec<Ident> {
    match expr {
        // For literals we dont have any unbound ident.
        Expr::Lit(_)
        | Expr::Continue(_)
        | Expr::Break(_) => {
            vec![]
        }
        // It is only a ident we care about if it has one and only one segment
        Expr::Path(i) => {
            if i.path.segments.len() == 1 {
                vec![i.path.segments[0].ident.clone()]
            } else {
                vec![]
            }
        }
        // Extract from the inner expression
        Expr::Box(i) => {
            extract_idents_from_expression(*i.expr.clone())
        }
        // Extract from the inner expression
        Expr::Group(i) => {
            extract_idents_from_expression(*i.expr.clone())
        }
        Expr::Binary(i) => {
            let mut res = HashSet::new();
            res.extend(extract_idents_from_expression(*i.left.clone()));
            res.extend(extract_idents_from_expression(*i.right.clone()));

            Vec::from_iter(res)
        }
        Expr::Array(i) => {
            let mut res = HashSet::new();
            for elem in i.elems {
                res.extend(extract_idents_from_expression(elem))
            }

            Vec::from_iter(res)
        }
        Expr::Await(i) => {
            extract_idents_from_expression(*i.base.clone())
        }
        Expr::Call(i) => {
            let mut res = HashSet::new();

            // Todo: Look at the ident from the func expression
            //res.extend(extract_idents_from_expression(*i.func.clone()));

            for elem in i.args {
                res.extend(extract_idents_from_expression(elem))
            }

            Vec::from_iter(res)
        }
        Expr::Cast(i) => {
            extract_idents_from_expression(*i.expr.clone())
        }
        Expr::Field(i) => {
            extract_idents_from_expression(*i.base.clone())
        }
        Expr::Index(i) => {
            let mut res = HashSet::new();
            res.extend(extract_idents_from_expression(*i.expr.clone()));
            res.extend(extract_idents_from_expression(*i.index.clone()));

            Vec::from_iter(res)
        }
        Expr::MethodCall(i) => {
            let mut res = HashSet::new();
            res.extend(extract_idents_from_expression(*i.receiver.clone()));

            for elem in i.args {
                res.extend(extract_idents_from_expression(elem))
            }

            Vec::from_iter(res)
        }
        Expr::Paren(i) => {
            extract_idents_from_expression(*i.expr.clone())
        }
        Expr::Range(i) => {
            let mut res = HashSet::new();
            if let Some(from) = i.from {
                res.extend(extract_idents_from_expression(*from.clone()));
            }
            if let Some(to) = i.to {
                res.extend(extract_idents_from_expression(*to.clone()));
            }

            Vec::from_iter(res)
        }
        Expr::Reference(i) => {
            extract_idents_from_expression(*i.expr.clone())
        }
        Expr::Repeat(i) => {
            let mut res = HashSet::new();
            res.extend(extract_idents_from_expression(*i.expr.clone()));
            res.extend(extract_idents_from_expression(*i.len.clone()));

            Vec::from_iter(res)
        }
        Expr::Return(i) => {
            if let Some(e) = i.expr {
                extract_idents_from_expression(*e.clone())
            } else {
                vec![]
            }
        }
        Expr::Struct(i) => {
            let mut res = HashSet::new();
            if let Some(rest) = i.rest {
                res.extend(extract_idents_from_expression(*rest.clone()));
            }

            for elem in i.fields {
                res.extend(extract_idents_from_expression(elem.expr))
            }

            Vec::from_iter(res)
        }
        Expr::Try(i) => {
            extract_idents_from_expression(*i.expr.clone())
        }
        Expr::Tuple(i) => {
            let mut res = HashSet::new();

            for elem in i.elems {
                res.extend(extract_idents_from_expression(elem))
            }

            Vec::from_iter(res)
        }
        Expr::Type(i) => {
            extract_idents_from_expression(*i.expr.clone())
        }
        Expr::Unary(i) => {
            extract_idents_from_expression(*i.expr.clone())
        }
        Expr::Macro(_) => {
            vec![] //Todo: try parse some common macros and get ident from then
        }
        _ => todo!("The type is not allowed within this context currently."),
        //Expr::Assign(_) => {}
        //Expr::AssignOp(_) => {}
        //Expr::Async(_) => {}
        //Expr::Block(_) => {}
        //Expr::Closure(_) => {}
        //Expr::ForLoop(_) => {}
        //Expr::If(_) => {}
        //Expr::Let(_) => {}
        //Expr::Loop(_) => {}
        //Expr::Match(_) => {}
        //Expr::TryBlock(_) => {}
        //Expr::Unsafe(_) => {}
        //Expr::Verbatim(_) => {}
        //Expr::While(_) => {}
        //Expr::Yield(_) => {}

        #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
        _ => { panic!("Pat is a new token not yet matched with") }
    }
}

mod tests {
    
    
    

    #[test]
    fn extract_from_number_lit() {
        // Arrange
        let expr: Expr = parse_quote!(
            1
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_string_lit() {
        // Arrange
        let expr: Expr = parse_quote!(
            "test"
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_path1() {
        // Arrange
        let expr: Expr = parse_quote!(
            x
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_path2() {
        // Arrange
        let expr: Expr = parse_quote!(
            test::x
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_boxed_ident() {
        // Arrange
        let expr: Expr = parse_quote!(
            box x
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_break() {
        // Arrange
        let expr: Expr = parse_quote!(
            break
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_continue() {
        // Arrange
        let expr: Expr = parse_quote!(
            continue
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_binary1() {
        // Arrange
        let expr: Expr = parse_quote!(
            1 + 2
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_binary2() {
        // Arrange
        let expr: Expr = parse_quote!(
            a + 2
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site())
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_binary3() {
        // Arrange
        let expr: Expr = parse_quote!(
            2 + a
        );
        println!("{:#?}", &expr);

        // Act
        let actual = extract_idents_from_expression(expr);

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site())
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_binary4() {
        // Arrange
        let expr: Expr = parse_quote!(
            a + b
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
            Ident::new("b", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_binary5() {
        // Arrange
        let expr: Expr = parse_quote!(
            a + a
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site())
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_binary6() {
        // Arrange
        let expr: Expr = parse_quote!(
            a + a + b + c + 23 * 1 - d
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
            Ident::new("b", Span::call_site()),
            Ident::new("c", Span::call_site()),
            Ident::new("d", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_array1() {
        // Arrange
        let expr: Expr = parse_quote!(
            []
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_array2() {
        // Arrange
        let expr: Expr = parse_quote!(
            [1, 2, 3]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_array3() {
        // Arrange
        let expr: Expr = parse_quote!(
            [x, 2, y]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_array4() {
        // Arrange
        let expr: Expr = parse_quote!(
            [x]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_await() {
        // Arrange
        let expr: Expr = parse_quote!(
            x.await
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_call() {
        // Arrange
        let expr: Expr = parse_quote!(
            hello(x, y)
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_cast1() {
        // Arrange
        let expr: Expr = parse_quote!(
            x as u32
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_cast2() {
        // Arrange
        let expr: Expr = parse_quote!(
            1 as u32
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_field1() {
        // Arrange
        let expr: Expr = parse_quote!(
            0.x
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_field2() {
        // Arrange
        let expr: Expr = parse_quote!(
            x.y
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_field3() {
        // Arrange
        let expr: Expr = parse_quote!(
            x.y.z
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_index1() {
        // Arrange
        let expr: Expr = parse_quote!(
            0[0]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_index2() {
        // Arrange
        let expr: Expr = parse_quote!(
            x[0]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_index3() {
        // Arrange
        let expr: Expr = parse_quote!(
            x[y]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_method1() {
        // Arrange
        let expr: Expr = parse_quote!(
            x.y()
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_method2() {
        // Arrange
        let expr: Expr = parse_quote!(
            x.y(z, w)
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("w", Span::call_site()),
            Ident::new("x", Span::call_site()),
            Ident::new("z", Span::call_site()),
        ];

        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_paren() {
        // Arrange
        let expr: Expr = parse_quote!(
            (a)
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site())
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_range1() {
        // Arrange
        let expr: Expr = parse_quote!(
            ..
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_range2() {
        // Arrange
        let expr: Expr = parse_quote!(
            a..
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site())
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_range3() {
        // Arrange
        let expr: Expr = parse_quote!(
            a..b
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
            Ident::new("b", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_range4() {
        // Arrange
        let expr: Expr = parse_quote!(
            b..a
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
            Ident::new("b", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_range5() {
        // Arrange
        let expr: Expr = parse_quote!(
            ..a
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_reference() {
        // Arrange
        let expr: Expr = parse_quote!(
            &a
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_repeat1() {
        // Arrange
        let expr: Expr = parse_quote!(
            [1; 1]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_repeat2() {
        // Arrange
        let expr: Expr = parse_quote!(
            [a; 1]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_repeat3() {
        // Arrange
        let expr: Expr = parse_quote!(
            [a; b]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
            Ident::new("b", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_repeat4() {
        // Arrange
        let expr: Expr = parse_quote!(
            [a; a]
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_return1() {
        // Arrange
        let expr: Expr = parse_quote!(
            return 1
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_return2() {
        // Arrange
        let expr: Expr = parse_quote!(
            return a
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("a", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_return3() {
        // Arrange
        let expr: Expr = parse_quote!(
            return
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_struct1() {
        // Arrange
        let expr: Expr = parse_quote!(
            Test {}
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_struct2() {
        // Arrange
        let expr: Expr = parse_quote!(
            Test { x }
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_struct3() {
        // Arrange
        let expr: Expr = parse_quote!(
            Test { y: x }
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_struct4() {
        // Arrange
        let expr: Expr = parse_quote!(
            Test { x: 1, y: 1 }
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_struct5() {
        // Arrange
        let expr: Expr = parse_quote!(
            Test { x: 1, y: z, ..w }
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("w", Span::call_site()),
            Ident::new("z", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_try1() {
        // Arrange
        let expr: Expr = parse_quote!(
            1?
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_try2() {
        // Arrange
        let expr: Expr = parse_quote!(
            x?
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_tuple1() {
        // Arrange
        let expr: Expr = parse_quote!(
            (1,)
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_tuple2() {
        // Arrange
        let expr: Expr = parse_quote!(
            (1, x, y)
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_type() {
        // Arrange
        let expr: Expr = parse_quote!(
            x: f64
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_unary1() {
        // Arrange
        let expr: Expr = parse_quote!(
            !x
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![
            Ident::new("x", Span::call_site()),
        ];
        assert_eq!(expected, actual)
    }

    #[test]
    fn extract_from_unary2() {
        // Arrange
        let expr: Expr = parse_quote!(
            !true
        );
        println!("{:#?}", &expr);

        // Act
        let mut actual = extract_idents_from_expression(expr);
        actual.sort();

        // Assert
        let expected: Vec<Ident> = vec![];
        assert_eq!(expected, actual)
    }
}