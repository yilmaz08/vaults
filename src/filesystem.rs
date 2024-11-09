use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyEntry, ReplyDirectory, Request
};
use std::ffi::OsStr;
use libc::ENOENT;
use std::time::{Duration, SystemTime};

enum FileOrDir<'a> {
    File(&'a mut File),
    Directory(&'a mut Directory)
}

pub struct Vault {
    pub root: Directory
}

#[derive(Clone)]
pub struct Directory {
    pub name: String,
    pub directories: Vec::<Directory>,
    pub files: Vec::<File>,

    pub ino: u64,
    pub size: u64,
    pub blocks: u64,
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub crtime: SystemTime,
    pub kind: FileType,
    pub perm: u16,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u32,
    pub flags: u32,
    pub blksize: u32,
}
#[derive(Clone)]
pub struct File {
    pub name: String,
    pub data: Vec::<u8>,
    
    pub ino: u64,
    pub size: u64,
    pub blocks: u64,
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub crtime: SystemTime,
    pub kind: FileType,
    pub perm: u16,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u32,
    pub flags: u32,
    pub blksize: u32,

}

impl Filesystem for Vault {
    fn readdir(&mut self, _req: &Request<'_>, ino: u64, fh: u64, offset: i64, mut reply: ReplyDirectory) {
        println!("readdir - ino:{} fh:{} offset:{}", ino, fh, offset);
        
        if offset > 0 {
            reply.error(ENOENT);
            return;
        }

        match self.root.find_ino(ino) {
            Some(FileOrDir::File(_)) => reply.error(ENOENT),
            None => reply.error(ENOENT),
            Some(FileOrDir::Directory(dir)) => {
                reply.add(dir.ino, 1 as i64, FileType::Directory, ".");
                reply.add(1, 2 as i64, FileType::Directory, "..");
                
                let items = dir.list_items();

                for i in 0..items.len() {
                    reply.add(items[i].0, (i+2) as i64, items[i].1, items[i].2.clone());
                }

                reply.ok();
            }
        };
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        println!("getattr - ino:{} fh:{:?}", ino, fh);
        
        let item = self.root.find_ino(ino);
        match self.root.find_ino(ino) {
            None => reply.error(ENOENT),
            Some(FileOrDir::File(item)) => {
                let attr = FileAttr {
                    ino: item.ino,
                    size: item.size,
                    blocks: item.blocks,
                    atime: item.atime,
                    mtime: item.mtime,
                    ctime: item.ctime,
                    crtime: item.crtime,
                    kind: item.kind,
                    perm: item.perm,
                    nlink: item.nlink,
                    uid: item.uid,
                    gid: item.gid,
                    rdev: item.rdev,
                    flags: item.flags,
                    blksize: item.blksize
                };

                reply.attr(&Duration::new(1,0), &attr);
            },
            Some(FileOrDir::Directory(item)) => {
                let attr = FileAttr {
                    ino: item.ino,
                    size: item.size,
                    blocks: item.blocks,
                    atime: item.atime,
                    mtime: item.mtime,
                    ctime: item.ctime,
                    crtime: item.crtime,
                    kind: item.kind,
                    perm: item.perm,
                    nlink: item.nlink,
                    uid: item.uid,
                    gid: item.gid,
                    rdev: item.rdev,
                    flags: item.flags,
                    blksize: item.blksize
                };

                reply.attr(&Duration::new(1,0), &attr);
            }
        };
    }

    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup - parent:{} name:{:?}", parent, name);
        match self.root.find_ino(parent) {
            Some(FileOrDir::File(_)) => reply.error(ENOENT),
            None => reply.error(ENOENT),
            Some(FileOrDir::Directory(dir)) => {
                match dir.find_file(name.to_str().unwrap().to_string()) {
                    None => reply.error(ENOENT),
                    Some(file) => {
                        let attr = FileAttr {
                            ino: file.ino,
                            size: file.size,
                            blocks: file.blocks,
                            atime: file.atime,
                            mtime: file.mtime,
                            ctime: file.ctime,
                            crtime: file.crtime,
                            kind: file.kind,
                            perm: file.perm,
                            nlink: file.nlink,
                            uid: file.uid,
                            gid: file.gid,
                            rdev: file.rdev,
                            flags: file.flags,
                            blksize: file.blksize
                        };

                        reply.entry(&Duration::new(1,0), &attr, 0);
                    }
                }
            }
        };
    }
}

impl Directory {
    fn find_file(&mut self, name: String) -> Option<&mut File> {
        for i in 0..self.files.len() {
            if self.files[i].name == name {
                return Some(&mut self.files[i]);
            }
        }
        return None;
    }

    fn list_items(&mut self) -> Vec<(u64, FileType, String)> {
        let mut items = Vec::<(u64, FileType, String)>::new();

        for i in 0..self.files.len() { items.push((self.files[i].ino, FileType::RegularFile, self.files[i].name.clone())); }
        for i in 0..self.directories.len() { items.push((self.directories[i].ino, FileType::Directory, self.directories[i].name.clone())); }

        return items;
    }

    fn find_ino(&mut self, ino: u64) -> Option<FileOrDir> {
        if self.ino == ino { return Some(FileOrDir::Directory(self)); }
        for i in 0..self.files.len() {
            if self.files[i].ino == ino {
                return Some(FileOrDir::File(&mut self.files[i]));
            }
        }
        for i in 0..self.directories.len() {
            if self.directories[i].ino == ino {
                return Some(FileOrDir::Directory(&mut self.directories[i]))
            }
            if self.directories[i].ino > ino {
                return self.directories[i-1].find_ino(ino);
            }
        }
        return None;
    }
}
