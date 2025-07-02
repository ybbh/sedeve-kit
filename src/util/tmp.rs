use std::path::Path;

pub fn tmp_dir<P: AsRef<Path>>(path:P) -> String {
    let mut p = std::env::temp_dir();
    p = p.join(path);
    let ret_path = p.to_str().unwrap().to_string();
    ret_path
}
