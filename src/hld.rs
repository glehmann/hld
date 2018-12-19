use fs2::FileExt;
use rayon::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

/// buffer size for the digest computation
const BUFFER_SIZE: usize = 1024 * 1024;

/// compute the digest of a file
fn file_digest(path: &PathBuf) -> io::Result<sha1::Digest> {
    let mut f = File::open(path)?;
    let mut buffer = [0; BUFFER_SIZE];
    let mut m = sha1::Sha1::new();
    loop {
        let size = f.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        m.update(&buffer[0..size]);
    }
    Ok(m.digest())
}

// /// print the file digests
// fn print_digests(paths: &[PathBuf]) -> io::Result<()> {
//     for path in paths {
//         let sha1 = file_digest(&path)?;
//         println!("{}  {}", sha1, path.display());
//     }
//     println!("{:?}", find_file_duplicates(paths));
//     Ok(())
// }

/// find the duplicates in the provided paths
fn find_file_duplicates(paths: &[PathBuf], caches: &[PathBuf]) -> io::Result<Vec<Vec<PathBuf>>> {
    // compute a map of the digests to the path with that digest
    let file_map = Arc::new(Mutex::new(HashMap::new()));
    let ino_map = Mutex::new(HashMap::new());
    let cache = update_cache(caches)?;
    paths
        .iter()
        .map(|path| (path, file_map.clone()))
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(path, file_map)| -> io::Result<()> {
            let path = path.clone();
            // don't hardlink empty files
            if fs::metadata(path)?.len() > 0 {
                let inode = inos(path)?;
                let digest = if let Some(digest) = cache.get(path) {
                    *digest
                } else {
                    // ino_map.get(&inode).unwrap_or_else(|| file_digest(&path)?)
                    let ino_digest = match ino_map.lock().unwrap().get(&inode) {
                        Some(v) => Some(*v),
                        None => None,
                    };
                    match ino_digest {
                        Some(v) => v,
                        None => file_digest(&path)?,
                    }
                };
                file_map
                    .lock()
                    .unwrap()
                    .entry(digest)
                    .or_insert_with(Vec::new)
                    .push(path.clone());
                ino_map.lock().unwrap().insert(inode, digest);
            }
            Ok(())
        })
        .collect::<Result<Vec<_>, _>>()?;
    // then just keep the paths with duplicates
    let res = file_map.lock().unwrap().clone();
    Ok(res
        .into_iter()
        .filter(|(_, v)| v.len() >= 2)
        .map(|(_, v)| v)
        .collect())
}

const CACHE_PATH: &str = "/tmp/hld.cache";

fn update_cache(paths: &[PathBuf]) -> io::Result<HashMap<PathBuf, sha1::Digest>> {
    let cache = if let Ok(cache_reader) = File::open(CACHE_PATH) {
        let foo: HashMap<PathBuf, sha1::Digest> =
            serde_json::from_reader(cache_reader).unwrap_or_default();
        foo
    } else {
        let foo: HashMap<PathBuf, sha1::Digest> = HashMap::new();
        foo
    };
    // remove dead entries
    let mut cache: HashMap<PathBuf, sha1::Digest> = cache
        .into_iter()
        .filter(|(path, _)| path.exists())
        .collect();
    // paths.iter().for_each(|path| cache.entry(path.clone()).or_insert_with(|| file_digest(&path)?));
    for path in paths {
        let entry = cache.entry(path.clone());
        if let Entry::Vacant(entry) = entry {
            let digest = file_digest(&path)?;
            entry.insert(digest);
        }
    }
    let output_file = File::create(CACHE_PATH)?;
    output_file.lock_exclusive()?;
    serde_json::to_writer_pretty(&output_file, &cache)?;
    output_file.unlock()?;
    Ok(cache)
}

/// find the duplicated files and replace them with hardlinks
pub fn hardlink_deduplicate(paths: &[PathBuf], caches: &[PathBuf]) -> io::Result<()> {
    let dups = find_file_duplicates(paths, caches)?;
    for dup in dups {
        file_hardlinks(&dup[0], &dup[1..])?;
    }
    Ok(())
}

fn file_hardlinks(path: &PathBuf, hardlinks: &[PathBuf]) -> io::Result<()> {
    let inode = inos(path)?;
    for hardlink in hardlinks {
        let hinode = inos(hardlink)?;
        if hinode != inode && hinode.0 == inode.0 {
            info!("{} -> {}", hardlink.display(), path.display());
            std::fs::remove_file(hardlink)?;
            std::fs::hard_link(path, hardlink)?;
        }
    }
    Ok(())
}

pub fn glob_to_files(paths: &Vec<String>) -> Result<Vec<PathBuf>, glob::PatternError> {
    Ok(paths
        .into_iter()
        .flat_map(|g| glob::glob(g).unwrap().into_iter().filter_map(|f| f.ok()))
        .map(|f| f.to_path_buf())
        .filter(|f| f.metadata().unwrap().file_type().is_file())
        .collect())
}

/// returns the inodes of the partition and of the file
fn inos(path: &PathBuf) -> io::Result<(u64, u64)> {
    let metadata = fs::metadata(path)?;
    Ok((metadata.st_dev(), metadata.ino()))
}
