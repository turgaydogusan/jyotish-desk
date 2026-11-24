#[test]
fn julday_of_j2000_epoch_is_2451545() {
    let jd = ephemeris::julday_gregorian(2000, 1, 1, 12.0);
    assert_eq!(jd, 2_451_545.0);
}
