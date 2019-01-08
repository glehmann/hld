use crate::error::*;
use crate::strategy::*;
use bincode;
use blake2_rfc::blake2b::Blake2b;
use fs2::FileExt;
use itertools::chain;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io;
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use std::os::unix::fs as ufs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::vec::Vec;

const DIGEST_BYTES: usize = 32;
type Digest = [u8; DIGEST_BYTES];

/// compute the digest of a file
fn file_digest(path: &Path) -> Result<Digest> {
    debug!("computing digest of {}", path.display());
    let mut file = fs::File::open(&path).with_path(&path)?;
    let mut hasher = Blake2b::new(DIGEST_BYTES);
    io::copy(&mut file, &mut hasher).with_path(&path)?;
    let mut hash: Digest = Default::default();
    hash.copy_from_slice(hasher.finalize().as_bytes());
    Ok(hash)
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
    paths: &'a [PathBuf],
    caches: &'a [PathBuf],
    dry_run: bool,
    cache_path: &Path,
    clear_cache: bool,
) -> Result<Vec<Vec<&'a PathBuf>>> {
    // compute a map of the digests to the path with that digest
    let ino_map = Mutex::new(HashMap::new());
    let cache = update_cache(caches, dry_run, cache_path, clear_cache)?;

    // get some metadata and filter out the empty files
    let mut path_inos: Vec<(&'a PathBuf, (u64, u64))> = Vec::new();
    for path in chain(caches, paths) {
        let metadata = fs::metadata(path).with_path(path)?;
        if metadata.len() > 0 {
            path_inos.push((path, inos_m(&metadata)));
        }
    }

    // compute the digests
    let digests = path_inos
        .par_iter()
        .map(|(path, inode)| -> Result<(&'a PathBuf, Digest)> {
            let ino_digest: Option<Digest> =
                ino_map.lock().unwrap().get(inode).and_then(|v| Some(*v));
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
        .collect::<Result<Vec<(&'a PathBuf, Digest)>>>()?;

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

fn update_cache(
    paths: &[PathBuf],
    dry_run: bool,
    cache_path: &Path,
    clear_cache: bool,
) -> Result<HashMap<PathBuf, Digest>> {
    // locking the cache
    let lock_path = cache_path.with_extension("lock");
    let lock_file = File::create(&lock_path).with_path(&lock_path)?;
    lock_file.lock_exclusive().with_path(&lock_path)?;

    let cache: HashMap<PathBuf, Digest> = if clear_cache {
        hashmap! {}
    } else {
        File::open(&cache_path)
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
        if !dry_run {
            let output_file = File::create(&cache_path).with_path(&cache_path)?;
            bincode::serialize_into(io::BufWriter::new(&output_file), &live_cache)?;
        }
    }

    // unlock the cache
    lock_file.unlock().with_path(&cache_path)?;

    Ok(new_digests)
}

/// find the duplicated files and replace them with hardlinks
pub fn hardlink_deduplicate(
    paths: &[PathBuf],
    caches: &[PathBuf],
    dry_run: bool,
    cache_path: &Path,
    clear_cache: bool,
    strategy: Strategy,
) -> Result<()> {
    let dups = find_file_duplicates(paths, caches, dry_run, cache_path, clear_cache)?;
    let mut dedup_size: u64 = 0;
    let mut dedup_files: usize = 0;
    for dup in dups {
        dedup_size += file_hardlinks(&dup[0], &dup[1..], dry_run, strategy)?;
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

fn file_hardlinks(
    path: &Path,
    hardlinks: &[&PathBuf],
    dry_run: bool,
    strategy: Strategy,
) -> Result<u64> {
    let metadata = fs::metadata(path).with_path(path)?;
    let inode = inos_m(&metadata);
    for hardlink in hardlinks {
        let hinode = inos(hardlink)?;
        if hinode != inode && hinode.0 == inode.0 {
            debug!(
                "{}ing {} and {}",
                strategy,
                path.display(),
                hardlink.display(),
            );
            let dest_metadata = fs::metadata(hardlink).with_path(hardlink)?;
            if !dry_run {
                std::fs::remove_file(hardlink).with_path(hardlink)?;
                match strategy {
                    Strategy::SymLink => ufs::symlink(path, hardlink).with_path(path)?,
                    Strategy::HardLink => fs::hard_link(path, hardlink).with_path(path)?,
                    Strategy::RefLink => reflink::reflink(path, hardlink).with_path(path)?,
                }
                restore_file_attributes(hardlink, &dest_metadata)?;
            }
        } else {
            debug!(
                "{} and {} are already {}ed",
                path.display(),
                hardlink.display(),
                strategy,
            );
        }
    }
    Ok(metadata.len() * hardlinks.len() as u64)
}

fn restore_file_attributes(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    let atime = filetime::FileTime::from_last_access_time(metadata);
    let mtime = filetime::FileTime::from_last_modification_time(metadata);
    filetime::set_symlink_file_times(path, atime, mtime).with_path(path)?;
    fs::set_permissions(path, metadata.permissions()).with_path(path)?;
    Ok(())
}

pub fn glob_to_files(globs: &[String]) -> Result<Vec<PathBuf>> {
    let res = globs
        .par_iter()
        .map(|glob| {
            let mut res = VecDeque::new();
            for path in glob::glob(glob).with_glob(glob)? {
                let path = path?;
                if path.metadata().with_path(&path)?.file_type().is_file() {
                    res.push_back(path);
                }
            }
            Ok(res)
        })
        .collect::<Result<Vec<VecDeque<PathBuf>>>>()?;
    Ok(res.iter().cloned().flat_map(|v| v).collect())
}

/// returns the inodes of the partition and of the file
fn inos(path: &Path) -> Result<(u64, u64)> {
    Ok(inos_m(&fs::metadata(path).with_path(path)?))
}

fn inos_m(metadata: &fs::Metadata) -> (u64, u64) {
    (metadata.st_dev(), metadata.ino())
}
