use url::Url;

pub enum PackwizPath {
    Http(Url),
    File(std::path::PathBuf),
}
