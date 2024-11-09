use fuser::{mount2, MountOption};
use std::time::{UNIX_EPOCH, SystemTime};

mod filesystem;

fn main() {
    let mountpoint = std::env::args().nth(1).unwrap();
    let file = filesystem::File {
        data: "lorem ipsum dolor sit amet".into(),
        ino: 2,
        name: "test.txt".to_string(),

        size: 4096,
        blocks: 1,
        atime: SystemTime::now(),
        mtime: SystemTime::now(),
        ctime: SystemTime::now(),
        crtime: SystemTime::now(),
        kind: fuser::FileType::RegularFile,
        perm: 0o644,
        nlink: 1,
        uid: 0,
        gid: 0,
        rdev: 0,
        flags: 0,
        blksize: 512,
    };
    let root = filesystem::Directory {
        name: "".to_string(),
        ino: 1,
        directories: vec![],
        files: vec![file],

        size: 4096,
        blocks: 1,
        atime: UNIX_EPOCH,
        mtime: UNIX_EPOCH,
        ctime: UNIX_EPOCH,
        crtime: UNIX_EPOCH,
        kind: fuser::FileType::Directory,
        perm: 0o755,
        nlink: 1,
        uid: 0,
        gid: 0,
        rdev: 0,
        flags: 0,
        blksize: 512,
    };
    let v = filesystem::Vault {
        root: root
    };
    mount2(v, &mountpoint, &[MountOption::RW, MountOption::AutoUnmount, MountOption::AllowOther]);
}

