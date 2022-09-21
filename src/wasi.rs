use crate::FileTime;
use std::fs::{self, File};
use std::io;
use std::os::wasi::io::AsRawFd;
use std::path::Path;
use std::time::SystemTime;

pub fn set_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    unsafe { set_times(p, atime, mtime, wasi::LOOKUPFLAGS_SYMLINK_FOLLOW, wasi::FSTFLAGS_MTIM | wasi::FSTFLAGS_ATIM) }
}

pub fn set_symlink_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    unsafe { set_times(p, atime, mtime, 0, wasi::FSTFLAGS_MTIM | wasi::FSTFLAGS_ATIM) }
}

pub fn set_file_mtime(p: &Path, mtime: FileTime) -> io::Result<()> {
    unsafe { set_times(p, FileTime::zero(), mtime, wasi::LOOKUPFLAGS_SYMLINK_FOLLOW, wasi::FSTFLAGS_MTIM) }
}

pub fn set_file_atime(p: &Path, atime: FileTime) -> io::Result<()> {
    unsafe { set_times(p, atime, FileTime::zero(), wasi::LOOKUPFLAGS_SYMLINK_FOLLOW, wasi::FSTFLAGS_ATIM) }
}

pub fn from_last_modification_time(meta: &fs::Metadata) -> FileTime {
    let duration = meta
        .modified()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    FileTime {
        seconds: duration.as_secs() as i64,
        nanos: duration.subsec_nanos() as u32,
    }
}

pub fn from_last_access_time(meta: &fs::Metadata) -> FileTime {
    let duration = meta
        .accessed()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    FileTime {
        seconds: duration.as_secs() as i64,
        nanos: duration.subsec_nanos() as u32,
    }
}

pub fn from_creation_time(meta: &fs::Metadata) -> Option<FileTime> {
    let duration = meta
        .created()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    Some(FileTime {
        seconds: duration.as_secs() as i64,
        nanos: duration.subsec_nanos() as u32,
    })
}

pub fn set_file_handle_times(
    f: &File,
    atime: Option<FileTime>,
    mtime: Option<FileTime>,
) -> io::Result<()> {
    let atime_ = if let Some(a) = atime { (a.seconds as u64)*(1e9 as u64) + a.nanos as u64} else { 0 };
    let mtime_ = if let Some(a) = mtime { (a.seconds as u64)*(1e9 as u64) + a.nanos as u64} else { 0 };
    let fstflags = if atime_ == 0 { 0u16 } else { wasi::FSTFLAGS_ATIM }
        | if mtime_ == 0 { 0u16 } else { wasi::FSTFLAGS_MTIM };
    unsafe {
        if let Err(e) = wasi::fd_filestat_set_times(
            f.as_raw_fd() as u32,
            atime_ as u64,
            mtime_ as u64,
            fstflags
        ) { return Err(io::Error::new(io::ErrorKind::Other, e.to_string())); }
    }
    Ok(())
}

unsafe fn set_times(
    p: &Path,
    atime: FileTime,
    mtime: FileTime,
    lookupflags: wasi::Lookupflags,
    fstflags: wasi::Fstflags
) -> io::Result<()> {
    let path_ = if let Some(path) = p.to_str() { path } else { "" };
    if let Err(e) = wasi::path_filestat_set_times(
        File::open(".")?.as_raw_fd() as u32,
        lookupflags,
        path_,
        (atime.seconds as u64)*(1e9 as u64) + atime.nanos as u64,
        (mtime.seconds as u64)*(1e9 as u64) + atime.nanos as u64,
        fstflags
    ) { return Err(io::Error::new(io::ErrorKind::Other, e.to_string())); }
    Ok(())
}

