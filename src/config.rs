use config::{Config, File};

lazy_static! {
    pub static ref SETTINGS: Config = {
        let mut settings = Config::default();
        settings.merge(File::with_name("Settings.toml")).unwrap();
        settings
    };
}
