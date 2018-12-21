use fs2::FileExt;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Mutex;
use std::vec::Vec;

/// buffer size for the digest computation
const BUFFER_SIZE: usize = 1024 * 1024;

/// compute the digest of a file
fn file_digest(path: &PathBuf) -> io::Result<sha1::Digest> {
    debug!("computing digest of {}", path.display());
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
    let ino_map = Mutex::new(HashMap::new());
    let cache = update_cache(caches)?;
    let res = paths
        .par_iter()
        .map(|path| -> io::Result<HashMap<_, _>> {
            if fs::metadata(&path)?.len() == 0 {
                Ok(hashmap! {})
            } else {
                let inode = inos(&path)?;
                let ino_digest: Option<sha1::Digest> = ino_map
                    .lock()
                    .unwrap()
                    .get(&inode)
                    .map_or(None, |v| Some(*v));
                let digest = if let Some(digest) = ino_digest {
                    digest
                } else {
                    let digest = if let Some(digest) = cache.get(path) {
                        *digest
                    } else {
                        file_digest(&path)?
                    };
                    ino_map.lock().unwrap().insert(inode, digest);
                    digest
                };
                Ok(hashmap! {digest => vec![path.clone()]})
            }
        })
        .reduce(
            || Ok(hashmap! {}),
            |a, b| {
                let mut tmp = b?;
                a?.into_iter().for_each(|(digest, paths)| {
                    tmp.entry(digest).or_insert_with(Vec::new).extend(paths)
                });
                Ok(tmp)
            },
        )?;
    // then just keep the paths with duplicates
    Ok(res
        .into_iter()
        .filter(|(_, v)| v.len() >= 2)
        .map(|(_, v)| v)
        .collect())
}

const CACHE_PATH: &str = "/tmp/hld.cache";

fn update_cache(paths: &[PathBuf]) -> io::Result<HashMap<PathBuf, sha1::Digest>> {
    let cache: HashMap<PathBuf, sha1::Digest> = File::open(CACHE_PATH).ok().map_or_else(
        || HashMap::new(),
        |reader| serde_json::from_reader(reader).unwrap_or_default(),
    );

    // remove dead entries
    let mut cache: HashMap<_, _> = cache
        .into_iter()
        .collect::<Vec<(_, _)>>()
        .par_iter()
        .cloned()
        .filter(|(path, _)| path.exists())
        .collect();
    // compute the digest for the entries not already there
    let new_digests = paths
        .par_iter()
        .map(|path| {
            let digest = if let Some(digest) = cache.get(path) {
                *digest
            } else {
                file_digest(&path)?
            };
            Ok((path.clone(), digest))
        })
        .collect::<io::Result<HashMap<_, _>>>()?;

    cache.extend(new_digests.clone());

    let output_file = File::create(CACHE_PATH)?;
    output_file.lock_exclusive()?;
    serde_json::to_writer_pretty(&output_file, &cache)?;
    output_file.unlock()?;

    Ok(new_digests)
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
