fn main() {
    let database_url = std::env::var("DATABASE_URL").unwrap();

    let postgres_url = url::Url::parse(&database_url).unwrap();
    println!("{}", postgres_url)
}
