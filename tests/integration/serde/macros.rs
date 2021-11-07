use serde::{Deserialize, Serialize};
use serde_test::{assert_de_tokens_error, assert_tokens, Configure, Token};
use time::macros::{datetime, declare_format_string};
use time::OffsetDateTime;

declare_format_string!(
    test_custom_format,
    "custom format: [year]-[month]-[day] [hour]:[minute]:[second] [offset_hour]:[offset_minute]"
);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct TestCustomFormat {
    #[serde(with = "test_custom_format")]
    dt: OffsetDateTime,
    #[serde(with = "test_custom_format::option")]
    option: Option<OffsetDateTime>,
}

#[test]
fn custom_serialize() {
    let value = TestCustomFormat {
        dt: datetime!(2000-01-01 00:00 -4:00),
        option: Some(datetime!(2000-01-01 00:00 -4:00)),
    };
    assert_tokens(
        &value.compact(),
        &[
            Token::Struct {
                name: "TestCustomFormat",
                len: 2,
            },
            Token::Str("dt"),
            Token::BorrowedStr("custom format: 2000-01-01 00:00:00 -04:00"),
            Token::Str("option"),
            Token::Some,
            Token::BorrowedStr("custom format: 2000-01-01 00:00:00 -04:00"),
            Token::StructEnd,
        ],
    );
    let value = TestCustomFormat {
        dt: datetime!(2000-01-01 00:00 -4:00),
        option: None,
    };
    assert_tokens(
        &value.compact(),
        &[
            Token::Struct {
                name: "TestCustomFormat",
                len: 2,
            },
            Token::Str("dt"),
            Token::BorrowedStr("custom format: 2000-01-01 00:00:00 -04:00"),
            Token::Str("option"),
            Token::None,
            Token::StructEnd,
        ],
    );
}

#[test]
fn custom_serialize_error() {
    // Deserialization error due to parse problem.
    assert_de_tokens_error::<TestCustomFormat>(
        &[
            Token::Struct {
                name: "TestCustomFormat",
                len: 2,
            },
            Token::Str("dt"),
            Token::BorrowedStr("not a date"),
            Token::StructEnd,
        ],
        "invalid value: literal, expected valid format",
    );
    // Parse problem in optional field.
    assert_de_tokens_error::<TestCustomFormat>(
        &[
            Token::Struct {
                name: "TestCustomFormat",
                len: 2,
            },
            Token::Str("dt"),
            Token::BorrowedStr("custom format: 2000-01-01 00:00:00 -04:00"),
            Token::Str("option"),
            Token::Some,
            Token::BorrowedStr("not a date"),
            Token::StructEnd,
        ],
        "invalid value: literal, expected valid format",
    );
}

use std::io;
struct BadWriter;

impl io::Write for BadWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, "oh no"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, "oh no"))
    }
}

#[test]
fn custom_serialize_io_error() {
    let value = TestCustomFormat {
        dt: datetime!(2000-01-01 00:00 -4:00),
        option: None,
    };

    let mut bytes = [0u8; 0];

    let assert_err = |res| {
        assert!(
            matches!(res, Err(time::error::Format::StdIo(e)) if e.kind() == std::io::ErrorKind::WriteZero)
        );
    };
    let mut ser = serde_json::Serializer::new(&mut BadWriter);
    assert_err(value.compact().serialize(&mut ser))
}