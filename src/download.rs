use std::{
    io::{Cursor},
};
use std::path::PathBuf;
use anyhow::anyhow;
use lapce_plugin::Http;


pub fn download_into(
    parent: PathBuf,
) -> Result<(), anyhow::Error> {
    if parent.exists() {
        std::fs::remove_dir_all(&parent)?;
    }
    std::fs::create_dir(&parent)?;
    let uri =
        "https://github.com/jm-observer/lapce-plugins/releases/download/0.0.1-lldb/lldb.zip".to_string();
    crate::info!("downloading {uri}");
    let mut resp = Http::get(&uri)?;
    let body = resp.body_read_all()?;
    let mut zip = zip::ZipArchive::new(Cursor::new(&body))?;
    crate::info!("downloaded {} {}", zip.len(), body.len());
    for i in 0..zip.len() {
        let mut zip_file = zip.by_index(i)?;
        let file_path = parent.join(zip_file.mangled_name());
        if zip_file.is_file() {
            let mut file = std::fs::File::create(&file_path).map_err(|x| {
                anyhow!("create file fail:{:?} {:?}", file_path, x)
            })?;
            std::io::copy(&mut zip_file, &mut file).map_err(|x| {
                anyhow!("copy file fail:{:?} {:?}", file_path, x)
            })?;
        } else {
            std::fs::create_dir(&file_path).map_err(|x| {
                anyhow!("create dir fail:{:?} {:?}", file_path, x)
            })?;
        }
    }
    Ok(())
}
