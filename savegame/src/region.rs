use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::{GzDecoder, ZlibDecoder};
use glam::IVec2;

pub struct RegionStorage {
    path: PathBuf,
    cache: HashMap<IVec2, Region>,
}

impl RegionStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        RegionStorage {
            path: {
                let mut path_ = PathBuf::new();
                path_.push(path);
                path_
            },
            cache: Default::default(),
        }
    }

    pub fn read(&mut self, position: IVec2) -> Option<Vec<u8>> {
        match self.cache.entry(position) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let path = self
                    .path
                    .join(format!("r.{}.{}.mca", position.x >> 5, position.y >> 5));
                if path.exists() {
                    entry.insert(Region::new(path))
                } else {
                    return None;
                }
            }
        }
        .read(((position.x & 0x1F) as u32 | ((position.y & 0x1F) as u32) << 5) as usize)
    }
}

struct Region {
    file: File,
    file_header: Vec<u8>,
}

impl Region {
    #[allow(clippy::unused_io_amount)]
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut file = File::open(path).unwrap();
        let mut file_header = vec![0; 4096 * 2];
        file.read(&mut file_header).unwrap();
        Region { file, file_header }
    }

    pub fn read(&mut self, index: usize) -> Option<Vec<u8>> {
        let location = (&self.file_header[index * 4..])
            .read_u32::<BigEndian>()
            .unwrap();
        let _timestamp = (&self.file_header[index * 4 + 4096..])
            .read_u32::<BigEndian>()
            .unwrap();
        if location == 0 {
            return None;
        }

        let sector_offset = ((location >> 8) * 4096) as usize;
        let sector_size = ((location & 0xFF) * 4096) as usize;
        if sector_offset < 2 {
            return None;
        }
        self.file
            .seek(SeekFrom::Start(sector_offset as u64))
            .unwrap();

        let size = self.file.read_u32::<BigEndian>().unwrap() as usize;
        if size > sector_size {
            return None;
        }

        let mut data = vec![0; size];
        self.file.read_exact(&mut data).unwrap();
        let mut data = &data[..];
        match data.read_u8().unwrap() {
            1 => {
                let mut decompressed_data = Vec::new();
                GzDecoder::new(&mut data)
                    .read_to_end(&mut decompressed_data)
                    .unwrap();
                Some(decompressed_data)
            }
            2 => {
                let mut decompressed_data = Vec::new();
                ZlibDecoder::new(&mut data)
                    .read_to_end(&mut decompressed_data)
                    .unwrap();
                Some(decompressed_data)
            }
            3 => Some(data.to_vec()),
            _ => None,
        }
    }
}
