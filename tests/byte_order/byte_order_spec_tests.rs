use qubit_codec::{BigEndian, ByteOrder, ByteOrderSpec, LittleEndian};

#[test]
fn test_byte_order_spec_exposes_runtime_order() {
    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    assert_eq!(ByteOrder::LittleEndian, LittleEndian::ORDER);
}
