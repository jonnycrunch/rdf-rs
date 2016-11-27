use uri::Uri;

// todo
pub enum XmlDataTypes {
  String,
  Decimal,
  Double,
  Boolean,
  Date,
  Long,
  Int,
  Integer,
}

impl XmlDataTypes {
  pub fn to_uri(&self) -> Uri {
    Uri::new(self.to_string())
  }

  pub fn to_string(&self) -> String {
    let schema_name = "http://www.w3.org/2001/XMLSchema#".to_string();

    // todo
    match *self {
      XmlDataTypes::Boolean => schema_name + "boolean",
      XmlDataTypes::Integer => schema_name + "integer",
      XmlDataTypes::Decimal => schema_name + "decimal",
      XmlDataTypes::Double => schema_name + "double",
      _ => "todo".to_string()
    }
  }
}

pub struct XmlSpecs { }

impl XmlSpecs {
  // todo

}