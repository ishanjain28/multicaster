use multicaster::Config;
fn main() {
    let config = Config::parse("config.toml").expect("error in parsing config");

    println!("{:?}", config);

    multicaster::start(config);
}
