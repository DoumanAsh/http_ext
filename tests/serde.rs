use http_ext::uri::{self, UriPathBuilder};
use std::collections::BTreeMap;

use serde::Serialize;

#[test]
fn should_serde_encode_query_from_map() {
    let mut query = BTreeMap::new();
    query.insert("key 1", "value 1");
    query.insert("key 2", "value 2");

    let mut builder = UriPathBuilder::new("/test", uri::PathEscape).join("ろり").with_query(uri::QueryEscape);
    builder.add_serde(&query).expect("to serialize");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/%E3%82%8D%E3%82%8A");
    assert_eq!(output.query(), Some("key%201=value%201&key%202=value%202"));

    query.clear();
    query.insert("key+1", "value+1");
    query.insert("key+2", "value+2");

    let mut builder = UriPathBuilder::new("/test", ()).join("ろり").with_query(());
    builder.add_serde(&query).expect("to serialize");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/ろり");
    assert_eq!(output.query(), Some("key+1=value+1&key+2=value+2"));
}

#[test]
fn should_serde_encode_query_from_seq() {
    let query = [("key 1", "value 1"), ("key 2", "value 2")];

    let mut builder = UriPathBuilder::new("/test", uri::PathEscape).join("ろり").with_query(uri::QueryEscape);
    builder.add_serde(&query).expect("to serialize");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/%E3%82%8D%E3%82%8A");
    assert_eq!(output.query(), Some("key%201=value%201&key%202=value%202"));

    let query = [("key+1", "value+1"), ("key+2", "value+2")];
    let mut builder = UriPathBuilder::new("/test", ()).join("ろり").with_query(());
    builder.add_serde(&query).expect("to serialize");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/ろり");
    assert_eq!(output.query(), Some("key+1=value+1&key+2=value+2"));
}

#[test]
fn should_serde_encode_query_from_tuple() {
    let query = (("key 1", "value 1"), ("key 2", "value 2"));

    let mut builder = UriPathBuilder::new("/test", uri::PathEscape).join("ろり").with_query(uri::QueryEscape);
    builder.add_serde(&query).expect("to serialize");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/%E3%82%8D%E3%82%8A");
    assert_eq!(output.query(), Some("key%201=value%201&key%202=value%202"));

    let query = (("key+1", "value+1"), ("key+2", "value+2"));
    let mut builder = UriPathBuilder::new("/test", ()).join("ろり").with_query(());
    builder.add_serde(&query).expect("to serialize");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/ろり");
    assert_eq!(output.query(), Some("key+1=value+1&key+2=value+2"));
}

#[test]
fn should_serde_encode_query_from_stuct() {
    struct MyStruct {
        key: &'static str,
        key2: &'static str,
    }

    impl Serialize for MyStruct {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            use serde::ser::SerializeStruct;

            let mut ser = serializer.serialize_struct("MyStruct", 2)?;
            ser.serialize_field("key", self.key)?;
            ser.serialize_field("key2", self.key2)?;
            ser.end()
        }
    }
    let query = MyStruct {
        key: "value 1",
        key2: "value 2",
    };

    let mut builder = UriPathBuilder::new("/test", uri::PathEscape).join("ろり").with_query(uri::QueryEscape);
    builder.add_serde(&query).expect("to serialize");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/%E3%82%8D%E3%82%8A");
    assert_eq!(output.query(), Some("key=value%201&key2=value%202"));

    let query = MyStruct {
        key: "value+1",
        key2: "value+2",
    };
    let mut builder = UriPathBuilder::new("/test", ()).join("ろり").with_query(());
    builder.add_serde(&query).expect("to serialize");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/ろり");
    assert_eq!(output.query(), Some("key=value+1&key2=value+2"));
}
