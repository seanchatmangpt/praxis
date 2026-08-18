// PASS: ReceiptRefusal variants are PartialEq + Eq, so assert_eq! works.

fn main() {
    use {{project-name}}::ReceiptRefusal;

    let e1 = ReceiptRefusal::SeqOutOfOrder { got: 5, expected: 4 };
    let e2 = ReceiptRefusal::SeqOutOfOrder { got: 5, expected: 4 };
    assert_eq!(e1, e2);

    assert_eq!(ReceiptRefusal::EmptyEventId, ReceiptRefusal::EmptyEventId);
}
