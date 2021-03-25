extern crate suzu;

use suzu::core::GensoDate;

#[test]
fn day_diff_works() {
    assert_eq!(GensoDate::new(112, 7, 23).diff_day(&GensoDate::new(112, 7, 23)), 0);
    assert_eq!(GensoDate::new(112, 7, 23).diff_day(&GensoDate::new(112, 8, 1)), 9);
    assert_eq!(GensoDate::new(112, 8, 1).diff_day(&GensoDate::new(112, 7, 23)), -9);
}
