pub fn year_to_season(year: i64) -> i64 {
    year
}

pub fn random_select<T>(mut i: std::slice::Iter<T>) -> Option<&T> {
    i.nth(rand::random::<usize>() % i.len())
}