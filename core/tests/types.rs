use rusty_php::test::TestBed;
use rusty_php::zend::Value;

#[test]
fn long() {
    let value = Value::from(TestBed::run_eval("1234500000 + 67890"));
    assert_eq!(value, Value::Long(1234567890));

    let value = Value::from(TestBed::run_eval("\\PHP_INT_MAX"));
    assert_eq!(value, Value::Long(i64::MAX));
}

#[test]
fn double() {
    let value = Value::from(TestBed::run_eval("1.234 + 5.678"));
    assert_eq!(value, Value::Double(6.912));
}

#[test]
fn string() {
    let value = Value::from(TestBed::run_eval("'Hello, world!'"));
    assert_eq!(value, Value::String("Hello, world!".as_bytes()));
}
