
# https://doc.rust-lang.org/cargo/reference/profiles.html#bench

rm -rf target/criterion
GNUPLOT_DEFAULT_TERM=png && cargo bench --bench criterion
