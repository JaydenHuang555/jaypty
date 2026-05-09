use env_logger::Builder;

pub fn main() {
    let mut b = Builder::new();
    b.filter_level(log::LevelFilter::Info);
    b.init();
}
