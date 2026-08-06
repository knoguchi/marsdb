#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(clippy::all)]

pub mod cypherlexer;
pub mod cypherparser;
pub mod cypherparserbaselistener;
pub mod cypherparserbasevisitor;
pub mod cypherparserlistener;
pub mod cypherparservisitor;

/// Phase 1 diagnostic only (mars-0mn) -- true if the vendored grammars-v4
/// grammar's `script` rule accepts `input` without a syntax error. Used to
/// empirically compare its acceptance rate against pest's, against the real
/// TCK corpus, before investing in Phase 2's clause-by-clause rewrite --
/// not a real parse path (no AST, no error detail). Delete once Phase 2's
/// visitor-based parser replaces it.
pub fn antlr_accepts(input: &str) -> bool {
    use antlr4rust::common_token_stream::CommonTokenStream;
    use antlr4rust::error_listener::ErrorListener;
    use antlr4rust::recognizer::Recognizer;
    use antlr4rust::token_factory::TokenFactory;
    use antlr4rust::InputStream;
    use antlr4rust::Parser;
    use cypherlexer::CypherLexer;
    use cypherparser::CypherParser;
    use std::cell::Cell;
    use std::rc::Rc;

    struct FlagOnError(Rc<Cell<bool>>);
    impl<'a, T: Recognizer<'a>> ErrorListener<'a, T> for FlagOnError {
        fn syntax_error(
            &self,
            _recognizer: &T,
            _offending_symbol: Option<&<T::TF as TokenFactory<'a>>::Inner>,
            _line: isize,
            _column: isize,
            _msg: &str,
            _e: Option<&antlr4rust::errors::ANTLRError>,
        ) {
            self.0.set(true);
        }
    }

    let had_error = Rc::new(Cell::new(false));
    let input = InputStream::new(input);
    let mut lexer = CypherLexer::new(input);
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(FlagOnError(had_error.clone())));
    let tokens = CommonTokenStream::new(lexer);
    let mut parser = CypherParser::new(tokens);
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(FlagOnError(had_error.clone())));
    let result = parser.script();
    result.is_ok() && !had_error.get()
}

#[cfg(test)]
mod phase1_spike {
    // Phase 1 proof-of-toolchain only (see ../../grammar/README.md) -- not
    // exercised by any real parsing path yet. Delete once Phase 2's visitor
    // AST builder has its own tests covering this ground.
    use super::cypherlexer::CypherLexer;
    use super::cypherparser::CypherParser;
    use antlr4rust::common_token_stream::CommonTokenStream;
    use antlr4rust::tree::ParseTree;
    use antlr4rust::InputStream;

    #[test]
    fn trivial_query_parses() {
        let input = InputStream::new("RETURN 1;");
        let lexer = CypherLexer::new(input);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let tree = parser.script().expect("trivial query should parse");
        assert_eq!(tree.get_text(), "RETURN1;<EOF>");
    }
}
