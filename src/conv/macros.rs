#[macro_export]
macro_rules! lazy_regex {
    ($($x:ident:$y:tt),*) => {
        $(pub static $x : LazyLock<Regex> = LazyLock::new(|| Regex::new($y).unwrap());)*};
}
