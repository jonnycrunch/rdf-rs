use Result;
use reader::rdf_parser::RdfParser;
use graph::Graph;
use error::{Error, ErrorType};
use triple::Triple;
use reader::lexer::turtle_lexer::TurtleLexer;
use reader::lexer::rdf_lexer::RdfLexer;
use node::Node;
use reader::lexer::token::Token;
use std::io::Read;
use uri::Uri;
use std::io::Cursor;
use namespace::Namespace;

/// RDF parser to generate an RDF graph from Turtle syntax.
pub struct TurtleParser<R: Read> {
  lexer: TurtleLexer<R>
}

impl<R: Read> RdfParser for TurtleParser<R> {
  /// Generates an RDF graph from a string containing Turtle syntax.
  ///
  /// Returns in error in case invalid Turtle syntax is provided.
  ///
  /// # Example
  ///
  /// todo
  ///
  fn decode(&mut self) -> Result<Graph> {
    let mut graph = Graph::new(None);

    loop {
      match self.lexer.peek_next_token() {
        Ok(Token::Comment(_)) => {
          let _ = self.lexer.get_next_token();
          continue
        },
        Ok(Token::EndOfInput) => return Ok(graph),
        Ok(Token::BaseDirective(base_uri)) => {
          graph.set_base_uri(&Uri::new(base_uri));
        },
        Ok(Token::PrefixDirective(prefix, uri)) => {
          graph.add_namespace(&Namespace::new(prefix, Uri::new(uri)));
        },
        Ok(Token::Uri(_)) | Ok(Token::BlankNode(_)) | Ok(Token::QName(_, _)) => {
          let triples = try!(self.read_triples(&graph));
          graph.add_triples(&triples);
        },
        Err(err) => {
          match err.error_type() {
            &ErrorType::EndOfInput(_) => return Ok(graph),
            error_type => return Err(Error::new(ErrorType::InvalidReaderInput,
                                                "Error while parsing Turtle syntax."))
          }
        }
        Ok(_) => return Err(Error::new(ErrorType::InvalidToken,
                                       "Invalid token while parsing Turtle syntax."))
      }
    }
  }
}

impl TurtleParser<Cursor<Vec<u8>>> {
  /// Constructor of `TurtleParser` from input string.
  pub fn from_string<S>(input: S) -> TurtleParser<Cursor<Vec<u8>>> where S: Into<String> {
    TurtleParser::from_reader(Cursor::new(input.into().into_bytes()))
  }
}


impl<R: Read> TurtleParser<R> {
  /// Constructor of `TurtleParser` from input reader.
  pub fn from_reader(input: R) -> TurtleParser<R> {
    TurtleParser {
      lexer: TurtleLexer::new(input)
    }
  }

  /// Creates a triple from the parsed tokens.
  fn read_triples(&mut self, graph: &Graph) -> Result<Vec<Triple>> {
    let mut triples: Vec<Triple> = Vec::new();

    let subject = try!(self.read_subject(&graph));
    let (predicate, object) = try!(self.read_predicate_with_object(graph));

    triples.push(Triple::new(&subject, &predicate, &object));

    loop {
      match self.lexer.get_next_token() {
        Ok(Token::TripleDelimiter) => break,
        Ok(Token::PredicateListDelimiter) => {
          let (predicate, object) = try!(self.read_predicate_with_object(graph));
          triples.push(Triple::new(&subject, &predicate, &object));
        },
        Ok(Token::ObjectListDelimiter) => {
          let object = try!(self.read_object(graph));
          triples.push(Triple::new(&subject, &predicate, &object));
        },
        _ => return Err(Error::new(ErrorType::InvalidReaderInput,
                                   "Invalid token while parsing Turtle triples."))
      }
    }

    Ok(triples)
  }

  /// Get the next token and check if it is a valid subject and create a new subject node.
  fn read_subject(&mut self, graph: &Graph) -> Result<Node> {
    match try!(self.lexer.get_next_token()) {
      Token::BlankNode(id) => Ok(Node::BlankNode { id: id }),
      Token::QName(prefix, path) => {
        let mut uri = try!(graph.get_namespace_uri_by_prefix(prefix)).to_owned();
        uri.append_resource_path(path.replace(":", "/"));   // adjust the QName path to URI path
        Ok(Node::UriNode { uri: uri })
      }
      Token::Uri(uri) => Ok(Node::UriNode { uri: Uri::new(uri) }),
      _ => Err(Error::new(ErrorType::InvalidToken,
                          "Invalid token for Turtle subject."))
    }
  }

  /// Get the next token and check if it is a valid predicate and create a new predicate node.
  fn read_predicate_with_object(&mut self, graph: &Graph) -> Result<(Node, Node)> {
    // read the predicate
    let predicate = match try!(self.lexer.get_next_token()) {
      Token::Uri(uri) => Node::UriNode { uri: Uri::new(uri) },
      Token::QName(prefix, path) => {
        let mut uri = try!(graph.get_namespace_uri_by_prefix(prefix)).to_owned();
        uri.append_resource_path(path.replace(":", "/"));   // adjust the QName path to URI path
        Node::UriNode { uri: uri }
      },
      _ => return Err(Error::new(ErrorType::InvalidToken, "Invalid token for Turtle predicate."))
    };

    // read the object
    let object = try!(self.read_object(graph));

    Ok((predicate, object))
  }

  /// Get the next token and check if it is a valid object and create a new object node.
  fn read_object(&mut self, graph: &Graph) -> Result<Node> {
    match try!(self.lexer.get_next_token()) {
      Token::BlankNode(id) => Ok(Node::BlankNode { id: id }),
      Token::Uri(uri) => Ok(Node::UriNode { uri: Uri::new(uri) }),
      Token::QName(prefix, path) => {
        let mut uri = try!(graph.get_namespace_uri_by_prefix(prefix)).to_owned();
        uri.append_resource_path(path.replace(":", "/"));   // adjust the QName path to URI path
        Ok(Node::UriNode { uri: uri })
      },
      Token::LiteralWithLanguageSpecification(literal, lang) =>
        Ok(Node::LiteralNode { literal: literal, data_type: None, language: Some(lang) }),
      Token::LiteralWithUrlDatatype(literal, datatype) =>
        Ok(Node::LiteralNode { literal: literal, data_type: Some(Uri::new(datatype)), language: None }),
      Token::Literal(literal) =>
        Ok(Node::LiteralNode { literal: literal, data_type: None, language: None }),
      _ => Err(Error::new(ErrorType::InvalidToken, "Invalid token for Turtle object."))
    }
  }
}