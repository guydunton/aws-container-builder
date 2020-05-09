#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl Tag {
    pub fn new(key: String, value: String) -> Tag {
        Tag { key, value }
    }
}

pub fn tags_validator(maybe_tag: String) -> Result<(), String> {
    tag_parser(maybe_tag).map(|_| ())
}

pub fn tag_parser(maybe_tag: String) -> Result<Vec<Tag>, String> {
    // Split by ,
    let tags: Vec<&str> = maybe_tag.split(",").collect();

    // Make sure no parts are empty
    if tags.iter().any(|tag| tag.len() == 0) {
        return Err("Tag must not be empty".to_owned());
    }

    let mut result = Vec::new();

    // Ensure each part can be split by =
    for tag in tags.iter() {
        match tag.split("=").collect::<Vec<&str>>()[..] {
            // Make sure there are 2 subparts
            [key, value] => {
                // Make sure no subparts are empty
                if key.len() == 0 {
                    return Err(format!("Key in tag ({}) must not be empty", tag));
                }

                if value.len() == 0 {
                    return Err(format!("Value in tag ({}) must not be empty", tag));
                }

                result.push(Tag::new(key.to_owned(), value.to_owned()));
            }
            _ => {
                return Err(format!("Tag ({}) must contain Key & Value", tag));
            }
        }
    }

    return Ok(result);
}

#[test]
fn tags_parser_should_error_if_comma_at_start_or_end() {
    assert!(tag_parser(",Key=Value".to_owned()).is_err());
    assert!(tag_parser("Key=Value,".to_owned()).is_err());
}

#[test]
fn tags_parser_should_error_if_key_or_value_is_empty() {
    assert!(tag_parser("=Value".to_owned()).is_err());
    assert!(tag_parser("Key=".to_owned()).is_err());
}

#[test]
fn tags_parser_should_error_if_tag_does_not_have_2_parts() {
    assert!(tag_parser("Key=Value=Error".to_owned()).is_err());
    assert!(tag_parser("Key".to_owned()).is_err());
}

#[test]
fn tags_parser_should_ok_with_single_tag() {
    assert_eq!(
        tag_parser("Key=Value".to_owned()),
        Ok(vec![Tag::new("Key".to_owned(), "Value".to_owned())])
    );
}

#[test]
fn tags_parser_should_ok_with_multiple_values() {
    assert_eq!(
        tag_parser("Key1=Value1,Key2=Value2".to_owned()),
        Ok(vec![
            Tag::new("Key1".to_owned(), "Value1".to_owned()),
            Tag::new("Key2".to_owned(), "Value2".to_owned())
        ])
    );
}
