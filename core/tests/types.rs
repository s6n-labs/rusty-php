extern crate core;

use rusty_php::test::TestBed;
use rusty_php::zend::Value;

#[test]
fn long() {
    let bed = TestBed::startup();
    let value = Value::from(bed.eval("1234500000 + 67890"));
    assert_eq!(value, Value::Long(1234567890));

    let value = Value::from(bed.eval("\\PHP_INT_MAX"));
    assert_eq!(value, Value::Long(i64::MAX));
}

#[test]
fn double() {
    let bed = TestBed::startup();
    let value = Value::from(bed.eval("1.234 + 5.678"));
    assert_eq!(value, Value::Double(6.912));
}

#[test]
fn string() {
    let bed = TestBed::startup();
    let value = Value::from(bed.eval("'Hello, world!'"));
    assert_eq!(value, Value::String("Hello, world!".into()));
}

#[test]
fn array() {
    let bed = TestBed::startup();
    let value = Value::from(bed.eval("[123, 4.56, 'Hello']"));
    let array = match value {
        Value::Array(a) => a,
        _ => panic!("not an array"),
    };

    assert_eq!(
        vec![
            Value::Long(123),
            Value::Double(4.56),
            Value::String("Hello".into()),
        ],
        array.into_iter().map(|e| (e.value())).collect::<Vec<_>>(),
    )
}
