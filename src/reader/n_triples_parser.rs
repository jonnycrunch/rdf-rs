use error::{Error, ErrorType};
use graph::Graph;
use node::Node;
use reader::lexer::n_triples_lexer::NTriplesLexer;
use reader::lexer::rdf_lexer::RdfLexer;
use reader::lexer::token::Token;
use reader::rdf_parser::RdfParser;
use std::io::Cursor;
use std::io::Read;
use triple::Triple;
use uri::Uri;
use Result;

/// RDF parser to generate an RDF graph from N-Triples syntax.
pub struct NTriplesParser<R: Read> {
    lexer: NTriplesLexer<R>,
}

impl<R: Read> RdfParser for NTriplesParser<R> {
    /// Generates an RDF graph from a string containing N-Triples syntax.
    ///
    /// Returns an error in case invalid N-Triples syntax is provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use rdf::reader::n_triples_parser::NTriplesParser;
    /// use rdf::reader::rdf_parser::RdfParser;
    ///
    /// let input = "<http://www.w3.org/2001/sw/RDFCore/ntriples/> <http://xmlns.com/foaf/0.1/maker> _:art .
    ///              _:art <http://xmlns.com/foaf/0.1/name> \"Art Barstow\" .";
    ///
    /// let mut reader = NTriplesParser::from_string(input.to_string());
    ///
    /// match reader.decode() {
    ///   Ok(graph) => assert_eq!(graph.count(), 2),
    ///   Err(_) => assert!(false)
    /// }
    /// ```
    ///
    /// # Failures
    ///
    /// - Invalid input that does not conform with NTriples standard.
    /// - Invalid node type for triple segment.
    ///
    fn decode(&mut self) -> Result<Graph> {
        let mut graph = Graph::new(None);

        loop {
            match self.lexer.peek_next_token()? {
                Token::Comment(_) => {
                    // ignore comments
                    let _ = self.lexer.get_next_token();
                    continue;
                }
                Token::EndOfInput => return Ok(graph),
                _ => {}
            }

            match self.read_triple() {
                Ok(triple) => graph.add_triple(&triple),
                Err(err) => match *err.error_type() {
                    ErrorType::EndOfInput(_) => return Ok(graph),
                    _ => {
                        println!("Error: {}", err.to_string());
                        return Err(Error::new(
                            ErrorType::InvalidReaderInput,
                            "Error while parsing NTriples syntax.",
                        ));
                    }
                },
            }
        }
    }
}

impl NTriplesParser<Cursor<Vec<u8>>> {
    /// Constructor of `NTriplesParser` from input string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rdf::reader::n_triples_parser::NTriplesParser;
    /// use rdf::reader::rdf_parser::RdfParser;
    ///
    /// let input = "<http://www.w3.org/2001/sw/RDFCore/ntriples/> <http://xmlns.com/foaf/0.1/maker> _:art .
    ///              _:art <http://xmlns.com/foaf/0.1/name> \"Art Barstow\" .";
    ///
    /// let reader = NTriplesParser::from_string(input.to_string());
    /// ```
    pub fn from_string<S>(input: S) -> NTriplesParser<Cursor<Vec<u8>>>
    where
        S: Into<String>,
    {
        NTriplesParser::from_reader(Cursor::new(input.into().into_bytes()))
    }
}

impl<R: Read> NTriplesParser<R> {
    /// Constructor of `NTriplesParser` from input reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use rdf::reader::n_triples_parser::NTriplesParser;
    /// use rdf::reader::rdf_parser::RdfParser;
    ///
    /// let input = "<http://www.w3.org/2001/sw/RDFCore/ntriples/> <http://xmlns.com/foaf/0.1/maker> _:art .
    ///              _:art <http://xmlns.com/foaf/0.1/name> \"Art Barstow\" .";
    ///
    /// let reader = NTriplesParser::from_reader(input.as_bytes());
    /// ```
    pub fn from_reader(input: R) -> NTriplesParser<R> {
        NTriplesParser {
            lexer: NTriplesLexer::new(input),
        }
    }

    /// Creates a triple from the parsed tokens.
    fn read_triple(&mut self) -> Result<Triple> {
        let subject = self.read_subject()?;
        let predicate = self.read_predicate()?;
        let object = self.read_object()?;

        match self.lexer.get_next_token() {
            Ok(Token::TripleDelimiter) => {}
            _ => {
                return Err(Error::new(
                    ErrorType::InvalidReaderInput,
                    "Expected triple delimiter.",
                ))
            }
        }

        Ok(Triple::new(&subject, &predicate, &object))
    }

    /// Get the next token and check if it is a valid subject and create a new subject node.
    fn read_subject(&mut self) -> Result<Node> {
        match self.lexer.get_next_token() {
            Ok(Token::BlankNode(id)) => Ok(Node::BlankNode { id }),
            Ok(Token::Uri(uri)) => Ok(Node::UriNode { uri: Uri::new(uri) }),
            _ => Err(Error::new(
                ErrorType::InvalidToken,
                "Invalid token for NTriples subject.",
            )),
        }
    }

    /// Get the next token and check if it is a valid predicate and create a new predicate node.
    fn read_predicate(&mut self) -> Result<Node> {
        match self.lexer.get_next_token() {
            Ok(Token::Uri(uri)) => Ok(Node::UriNode { uri: Uri::new(uri) }),
            _ => Err(Error::new(
                ErrorType::InvalidToken,
                "Invalid token for NTriples predicate.",
            )),
        }
    }

    /// Get the next token and check if it is a valid object and create a new object node.
    fn read_object(&mut self) -> Result<Node> {
        match self.lexer.get_next_token()? {
            Token::BlankNode(id) => Ok(Node::BlankNode { id }),
            Token::Uri(uri) => Ok(Node::UriNode { uri: Uri::new(uri) }),
            Token::LiteralWithLanguageSpecification(literal, lang) => Ok(Node::LiteralNode {
                literal,
                data_type: None,
                language: Some(lang),
            }),
            Token::LiteralWithUrlDatatype(literal, datatype) => Ok(Node::LiteralNode {
                literal,
                data_type: Some(Uri::new(datatype)),
                language: None,
            }),
            Token::Literal(literal) => Ok(Node::LiteralNode {
                literal,
                data_type: None,
                language: None,
            }),
            _ => Err(Error::new(
                ErrorType::InvalidToken,
                "Invalid token for NTriples object.",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use reader::n_triples_parser::NTriplesParser;
    use reader::rdf_parser::RdfParser;

    #[test]
    fn test_read_n_triples_from_string() {
        let input = "<http://www.w3.org/2001/sw/RDFCore/ntriples/> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Document> .
                 <http://www.w3.org/2001/sw/RDFCore/ntriples/> <http://purl.org/dc/terms/title> \"N-Triples\"@en-US .
                 <http://www.w3.org/2001/sw/RDFCore/ntriples/> <http://xmlns.com/foaf/0.1/maker> _:art .
                 _:art <http://xmlns.com/foaf/0.1/name> \"Art Barstow\" .";

        let mut reader = NTriplesParser::from_string(input.to_string());

        match reader.decode() {
            Ok(graph) => assert_eq!(graph.count(), 4),
            Err(e) => {
                println!("Err {}", e.to_string());
                assert!(false)
            }
        }
    }
}
