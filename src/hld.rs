use crate::cli::*;
use crate::error::{GlobResultExt, IOResultExt, Result};
use crate::strategy::Strategy;
use blake3::{Hash, Hasher};
use file_id::*;
use fs2::FileExt;
use itertools::chain;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::vec::Vec;

/// compute the digest of a file
fn file_digest(path: &Path) -> Result<Hash> {
    debug!("computing digest of {}", path.display());
    let mut file = fs::File::open(path).path_ctx(path)?;
    let mut hasher = Hasher::new();
    io::copy(&mut file, &mut hasher).path_ctx(path)?;
    Ok(hasher.finalize())
}

// /// print the file digests
// fn print_digests(paths: &[PathBuf]) -> Result<()> {
//     for path in paths {
//         let sha1 = file_digest(&path)?;
//         println!("{}  {}", sha1, path.display());
//     }
//     println!("{:?}", find_file_duplicates(paths));
//     Ok(())
// }

/// find the duplicates in the provided paths
fn find_file_duplicates<'a>(
    config: &Config,
    paths: &'a [PathBuf],
    caches: &'a [PathBuf],
) -> Result<Vec<Vec<&'a PathBuf>>> {
    // compute a map of the digests to the path with that digest
    let ino_map = Mutex::new(HashMap::new());
    let cache = update_cache(config, caches)?;

    // get some metadata and filter out the empty files
    let mut path_inos: Vec<(&'a PathBuf, FileId)> = Vec::new();
    for path in chain(caches, paths) {
        let metadata = fs::metadata(path).path_ctx(path)?;
        if metadata.len() > 0 {
            path_inos.push((path, get_file_id(path).path_ctx(path)?));
        }
    }

    // compute the digests
    let digests = path_inos
        .par_iter()
        .map(|(path, inode)| -> Result<(&'a PathBuf, Hash)> {
            let ino_digest: Option<Hash> = ino_map.lock().unwrap().get(inode).copied();
            let digest = if let Some(digest) = ino_digest {
                digest
            } else {
                let digest = if let Some(digest) = cache.get(*path) {
                    *digest
                } else {
                    file_digest(path)?
                };
                ino_map.lock().unwrap().insert(*inode, digest);
                digest
            };
            Ok((path, digest))
        })
        .collect::<Result<Vec<(&'a PathBuf, Hash)>>>()?;

    // merge the digests in a hashmap
    let mut res = hashmap! {};
    for (path, digest) in digests {
        res.entry(digest).or_insert_with(Vec::new).push(path);
    }

    // then just keep the paths with duplicates
    Ok(res
        .into_iter()
        .filter(|(_, v)| v.len() >= 2)
        .map(|(_, v)| v)
        .collect())
}

fn update_cache(config: &Config, paths: &[PathBuf]) -> Result<HashMap<PathBuf, Hash>> {
    // locking the cache
    let cache_dir = config.cache_path.parent().unwrap().to_owned();
    fs::create_dir_all(&cache_dir).path_ctx(&cache_dir)?;
    let lock_path = config.cache_path.with_extension("lock");
    let lock_file = File::create(&lock_path).path_ctx(&lock_path)?;
    lock_file.lock_exclusive().path_ctx(&lock_path)?;

    let cache: HashMap<PathBuf, Hash> = if config.clear_cache {
        hashmap! {}
    } else {
        File::open(&config.cache_path)
            .ok()
            .map_or_else(HashMap::new, |reader| {
                debug!("reading cache");
                bincode::deserialize_from(io::BufReader::new(reader)).unwrap_or_default()
            })
    };
    let original_cache_size = cache.len();

    // remove dead entries
    let mut live_cache: HashMap<_, _> = cache
        .into_iter()
        .collect::<Vec<(_, _)>>()
        .par_iter()
        .cloned()
        .filter(|(path, _)| path.exists())
        .collect();
    let live_cache_size = live_cache.len();
    let updated = original_cache_size != live_cache_size;

    // compute the digest for the entries not already there
    let new_digests = paths
        .par_iter()
        .map(|path| {
            let digest = live_cache
                .get(path)
                .map_or_else(|| file_digest(path), |d| Ok(*d))?;
            Ok((path.clone(), digest))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    live_cache.extend(new_digests.clone());
    let updated = updated || live_cache_size != live_cache.len();

    if updated {
        debug!("saving updated cache with {} entries", live_cache.len());
        if !config.dry_run {
            let output_file = File::create(&config.cache_path).path_ctx(&config.cache_path)?;
            bincode::serialize_into(io::BufWriter::new(&output_file), &live_cache)?;
        }
    }

    // unlock the cache
    FileExt::unlock(&lock_file).path_ctx(&config.cache_path)?;

    Ok(new_digests)
}

/// find the duplicated files and replace them with hardlinks
pub fn hardlink_deduplicate(config: &Config, paths: &[PathBuf], caches: &[PathBuf]) -> Result<()> {
    let dups = find_file_duplicates(config, paths, caches)?;
    let mut dedup_size: u64 = 0;
    let mut dedup_files: usize = 0;
    for dup in dups {
        dedup_size += file_hardlinks(config, dup[0], &dup[1..])?;
        dedup_files += dup.len() - 1;
    }
    debug!("{} bytes saved", dedup_size);
    debug!("{} files deduplicated", dedup_files);
    info!(
        "{} saved in the deduplication of {} files",
        pretty_bytes::converter::convert(dedup_size as f64),
        dedup_files
    );
    Ok(())
}

#[cfg(unix)]
fn crossplatform_symlink(path: &Path, hardlink: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(path, hardlink)
}

#[cfg(windows)]
fn crossplatform_symlink(path: &Path, hardlink: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(path, hardlink)
}

fn get_device_id(file_id: FileId) -> u64 {
    match file_id {
        FileId::Inode {
            device_id,
            inode_number: _,
        } => device_id,
        FileId::LowRes {
            volume_serial_number,
            file_index: _,
        } => volume_serial_number as u64,
        FileId::HighRes {
            volume_serial_number,
            file_id: _,
        } => volume_serial_number,
    }
}

fn file_hardlinks(config: &Config, path: &Path, hardlinks: &[&PathBuf]) -> Result<u64> {
    let metadata = fs::metadata(path).path_ctx(path)?;
    let inode = get_file_id(path).path_ctx(path)?;
    for hardlink in hardlinks {
        let hinode = get_file_id(hardlink).path_ctx(hardlink)?;
        if hinode != inode && get_device_id(hinode) == get_device_id(inode) {
            debug!(
                "{}ing {} and {}",
                config.strategy,
                path.display(),
                hardlink.display(),
            );
            let dest_metadata = fs::metadata(hardlink).path_ctx(hardlink)?;
            if !config.dry_run {
                std::fs::remove_file(hardlink).path_ctx(hardlink)?;
                match config.strategy {
                    Strategy::SymLink => crossplatform_symlink(path, hardlink).path_ctx(path)?,
                    Strategy::HardLink => fs::hard_link(path, hardlink).path_ctx(path)?,
                    Strategy::RefLink => reflink_copy::reflink(path, hardlink).path_ctx(path)?,
                }
                restore_file_attributes(hardlink, &dest_metadata)?;
            }
        } else {
            debug!(
                "{} and {} are already {}ed",
                path.display(),
                hardlink.display(),
                config.strategy,
            );
        }
    }
    Ok(metadata.len() * hardlinks.len() as u64)
}

fn restore_file_attributes(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    let atime = filetime::FileTime::from_last_access_time(metadata);
    let mtime = filetime::FileTime::from_last_modification_time(metadata);
    filetime::set_symlink_file_times(path, atime, mtime).path_ctx(path)?;
    fs::set_permissions(path, metadata.permissions()).path_ctx(path)?;
    Ok(())
}

pub fn glob_to_files(globs: &[String]) -> Result<Vec<PathBuf>> {
    let res = globs
        .par_iter()
        .map(|glob| {
            let mut res = VecDeque::new();
            for path in glob::glob(glob).glob_ctx(glob)? {
                let path = path?;
                if path
                    .symlink_metadata()
                    .path_ctx(&path)?
                    .file_type()
                    .is_file()
                {
                    res.push_back(path);
                }
            }
            Ok(res)
        })
        .collect::<Result<Vec<VecDeque<PathBuf>>>>()?;
    let mut res: Vec<PathBuf> = res.iter().flatten().cloned().collect();
    res.par_sort();
    res.dedup();
    Ok(res)
}
