pub(crate) mod attribute;
pub(crate) mod child;
pub(crate) mod io;
pub(crate) mod options;
pub(crate) mod spawn;

pub(crate) use options::CREATION_FLAGS;
pub(crate) use options::resolve_cmd;
pub(crate) use options::resolve_cwd;

pub(crate) use attribute::factory_attributes;

pub(crate) use io::cin;
pub(crate) use io::cout;

pub(crate) use child::watch;
pub use spawn::ContpySpawn;
