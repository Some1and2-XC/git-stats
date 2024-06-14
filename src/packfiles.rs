// https://dev.to/calebsander/git-internals-part-2-packfiles-1jg8

use core::fmt;
use std::{fs::File, io::{self, Read, Seek, SeekFrom}, path::PathBuf, str::FromStr};

use flate2::read::ZlibDecoder;
use regex::Regex;
use colored::Colorize;

use anyhow::{anyhow, Result};
use log::{self, debug};

use crate::objects::GitObjectType;

// The most significant bit of a 32 bit int.
// Used to see if the pack file uses 64 bit offsets.
const LONG_OFFSET_FLAG: u32 = 1 << 31;
const HASH_SIZE: usize = 20;

trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

/// Struct for emulating file system files 
/// with added functionality.
pub struct BytesFile {
    data: Box<dyn ReadSeek>,
}

impl BytesFile {
    fn from_path(path: &PathBuf) -> Result<Self> {
        return Ok(Self {
            data: Box::new(File::open(path)?),
        });
    }

    fn read_bytes<const N: usize>(&mut self) -> io::Result<[u8;N]> {
        let mut bytes = [0;N];
        self.data.read_exact(&mut bytes)?;
        return Ok(bytes);
    }

    fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_bytes()?;
        return Ok(u32::from_be_bytes(bytes));
    }

    fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_bytes()?;
        return Ok(u64::from_be_bytes(bytes));
    }

    fn read_hash(&mut self) -> Result<Hash> {
        let bytes = self.read_bytes()?;
        return Ok(Hash(bytes));
    }
}

#[derive(Debug)]
enum PackObjectType {
    Base(GitObjectType),
    OffsetDelta,
    HashDelta,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Hash(pub [u8;HASH_SIZE]);

impl Hash {
    pub fn to_string(&self) -> String {
        const HEX_VALUES: [char;16] = [
            '0','1','2','3',
            '4','5','6','7',
            '8','9','a','b',
            'c','d','e','f'];

        return self.0
            .iter()
            .map(|idx| {
                // Each byte is a u8
                // this means however that each value being iterated on is actually two characters
                // in utf-8
                let left = idx >> 4; // Does bit shifting to filter
                let right = idx & b'\x0f'; // Filters value for first 4 values

                // Returns left & right
                return format!(
                    "{}{}",
                    HEX_VALUES[left as usize],
                    HEX_VALUES[right as usize],
                    );
            }).collect();
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
    }
}

/// Represents an index packfile
pub struct Idx {
    pub version: u32,
    pub file: BytesFile,
}

impl Idx {
    // Creates a new Idx file from path.
    pub fn from_path(path: &PathBuf) -> Result<Self> {
        let mut file = BytesFile::from_path(path)?;
        let mut header = [0u8; 4];
        file.data.read_exact(&mut header)?;

        assert_eq!(header, *b"\xfftOc"); // tOc = table of contents (I think)

        let mut version_buf = [0u8; 4];
        file.data.read_exact(&mut version_buf)?;
        let version = u32::from_be_bytes(version_buf);

        log::debug!(
            "{} - version {version}",
            String::from_utf8_lossy(&header),
             );

        if version != 2 {
            log::debug!("Version of packfile index isn't '2', is '{version}' (may be unimplemented.)");
        }

        return Ok(Self {
            file,
            version,
        });
    }

    /*
    pub fn run(&mut self) -> Result<()> {
        self.file.data.seek(SeekFrom::Start(8))?;

        // The cumulative amount of objects up to that bin
        // (each bin refering to basically the foldername)
        let cumulative_objects: Vec<u32> = (0..=u8::MAX)
            .map(|_| self.file.read_u32().unwrap())
            .collect()
            ;

        let mut hashes: Vec<Hash> = vec![];

        let mut prev_objects = 0;
        // Remember, idx is the first byte of the hash.
        // It also is what the folder name is in git.
        for (idx, &obj_count) in cumulative_objects.iter().enumerate() {
            // let mut previous_hash = None;
            for _ in 0..(obj_count - prev_objects) {
                // The first byte is going to be the same as the index.
                hashes.push(self.file.read_hash()?);
            }
            prev_objects = obj_count;
        }

        let total_objects = hashes.len();
        info!("Total Objects: {total_objects}");
        assert_eq!(total_objects, cumulative_objects.last().unwrap().clone() as usize);

        // Ignores the hashes for all the files.
        for _ in 0..total_objects {
            let _crc32_object_hash = self.file.read_u32()?;
        }

        let mut pack_offsets_lst: Vec<u32> = vec![];

        let mut long_offsets = 0;
        // Reads the file offsets.
        for _ in 0..total_objects {
            let pack_offsets = self.file.read_u32()?;
            pack_offsets_lst.push(pack_offsets);
            if pack_offsets & LONG_OFFSET_FLAG != 0 {
                let offset_index = pack_offsets & !LONG_OFFSET_FLAG;
                long_offsets = long_offsets.max(offset_index + 1);
            }
        }

        return Ok(());
    }
    */

    /// Seeks 'offset' from file but skips the first two encoding bytes.
    fn seek_without_headers(&mut self, offset: u64) -> Result<()> {
        // Skips magic byte, version and sets offset.
        self.file.data.seek(SeekFrom::Start(
            4 + 4 +
            offset
            ))?;
        return Ok(());
    }

    /// Seeks 'offset' from file but skips the first two encoding bytes and the entire hash lookup table.
    fn seek_without_index(&mut self, offset: u64) -> Result<()> {
        // Skips the cumulative object counts and the previous hashes.
        self.file.data.seek(SeekFrom::Start(
            // Skips magic byte, version and sets offset.
            4 + 4 +
            offset * (HASH_SIZE as u64) + // skips previous values
            (u8::MAX as u64 * 4) + // skips lookup table
            4, // Honestly idek
            ))?;

        return Ok(());
    }

    /// Gets the upper and lower bounds of where a hash could be from index file.
    fn get_object_bounds(&mut self, hash: &Hash) -> Result<(u32, u32)> {
        let first_hash_byte = hash.0[0];
        let index_lower_bound = if first_hash_byte == 0 {
            self.seek_without_headers(0)?;
            0
        } else {
            self.seek_without_headers(first_hash_byte as u64 * 4 - 4)?;
            self.file.read_u32()?
        };
        let index_upper_bound = self.file.read_u32()?;
        return Ok((index_lower_bound, index_upper_bound));
    }

    fn get_object_index(&mut self, hash: Hash) -> Result<Option<u32>> {
        use std::cmp::Ordering::*;
        let (mut left, mut right) = self.get_object_bounds(&hash)?;
        debug!("B-Tree Searching for Hash: {hash}! left: {left} right: {right}");

        // Does binary search
        while left < right {
            let middle = left + (right - left) / 2;
            self.seek_without_index(middle as u64)?;
            let mid_hash = self.file.read_hash()?;

            match hash.cmp(&mid_hash) {
                Less => {
                    debug!("{} {mid_hash} {left} {right}", "<-".red());
                    right = middle
                },
                Equal => return Ok(Some(middle)),
                Greater => {
                    debug!("{} {mid_hash} {left} {right}", "->".green());
                    left = middle + 1
                },
            }
        }
        debug!("Didn't find hash in packfile!");
        Ok(None)
    }

    fn get_pack_offset_at_index(&mut self, offset: u32) -> Result<u64> {
        // Gets the total amount of objects
        self.seek_without_headers(u8::MAX as u64 * 4)?;
        let total_object_count = self.file.read_u32()?;
        debug!("Finding file '{offset}' of '{total_object_count}'");

        let crc_32_bytes = total_object_count * (HASH_SIZE as u32 + 4);
        self.seek_without_headers((crc_32_bytes + offset * 4) as u64).unwrap();

        let pack_offset = self.file.read_u32().unwrap();
        // If uses long offsets, read long offsets.
        if pack_offset & LONG_OFFSET_FLAG == 0 {
            debug!("Unpacking offset value without long offset flag...");
            return Ok(pack_offset as u64);
        } else {
            let offset_index = pack_offset & !LONG_OFFSET_FLAG;
            self.seek_without_headers(
                (crc_32_bytes +
                offset * 8) as u64,
                )?;
            debug!("Unpacking offset value with long offset flag...");
            return self.file.read_u64();
        }
    }
}

pub struct Pack {
    pub path: PathBuf,
    pub object_name: String,
    pub file: BytesFile,
}

impl Pack {
    pub fn from_path(path: &str) -> Result<Self> {
        let path_buf = PathBuf::from_str(path)?;
        if !path_buf.is_file() {
            return Err(anyhow!("Can't find packfile specified '{path}'."));
        } else if !path.ends_with(".pack") {
            return Err(anyhow!("Packfile '{path}' doesn't end with '.pack' (are you sure this is a valid packfile?)"));
        } else {
            let re = Regex::new(r"(?<index>pack-[0-9a-f]{40}).(idx|pack|rev)").unwrap();

            let captures = match re.captures(path) {
                Some(v) => v,
                None => return Err(anyhow!("Can't match filename '{}'!", path)),
            };

            let filename = captures.name("index").unwrap().as_str().to_owned();

            return Ok(
                Self {
                    path: path_buf,
                    object_name: filename,
                    file: BytesFile::from_path(&PathBuf::from_str(path)?)?,
                });
        }
    }

    /// Tries to get the pack object from hash.
    pub fn get_pack_offset(&mut self, hash: Hash) -> Result<Option<u64>> {
        let mut index = Idx::from_path(
            &self.path
                .parent()
                .unwrap()
                .join(self.object_name.to_owned() + ".idx")
            )?;
        let object_index = match index.get_object_index(hash)? {
            Some(v) => v,
            None => return Ok(None),
        };

        let pack_offset = index.get_pack_offset_at_index(object_index).unwrap();
        return Ok(Some(pack_offset));
    }

    // Returns the amount of encoding bits used
    const fn get_encoding_bits() -> u8 {
        return 7;
    }

    // Reads 7 bits and flag for if there are more values
    pub fn read_variant_byte(&mut self) -> Result<(u8, bool)> {
        // Meaning there are 7 bits for values
        const VARIANT_ENCODING_BITS: u8 = Pack::get_encoding_bits();

        // Gets the first bit which is the continue flag
        const VARIANT_ENCODING_CONTINUE_FLAG: u8 = 1 << VARIANT_ENCODING_BITS;

        let [byte] = self.file.read_bytes().unwrap();
        let value = byte & !VARIANT_ENCODING_CONTINUE_FLAG; // Gets the value without continue bit
        let more_bytes = byte & VARIANT_ENCODING_CONTINUE_FLAG != 0; // Gets continue flag

        return Ok((value, more_bytes));
    }

    pub fn read_size_encoding(&mut self) -> Result<usize> {
        let mut value = 0;
        let mut length = 0; // The total number of bits read
        const VARIANT_ENCODING_BITS: u8 = Pack::get_encoding_bits();

        loop {
            let (bytes_value, more_bytes) = self.read_variant_byte().unwrap();

            // Adds bytes to the output 'value' var
            // Note this does reverse the order of bytes read
            value |= (bytes_value as usize) << length;

            if !more_bytes {
                return Ok(value);
            }

            length += VARIANT_ENCODING_BITS;
        }
    }

    pub fn read_type_and_size(&mut self, offset: u64) -> Result<(u8, usize)> {

        self.file.data.seek(SeekFrom::Start(offset))?;
        let v = self.read_size_encoding()?;

        // the data is s..ssstttxxxx
        // where
        // ssss is object_size (can be any length including 0)
        // ttt is object_type (allways 3 bits)
        // xxxx is (I'm not exactly sure yet and always 4 bits)

        let object_first_four = v & 0b1111;
        let object_type = (v >> 4) & 0b111;
        // let object_size = (v >> 7) | (v >> 7 << 4);
        let object_size = v >> 7;

        debug!("Type and Size: '{v:b}' -> Type: '{object_type}' & Size: '{object_size}'");

        return Ok((object_type as u8, object_size));
    }

    fn read_pack_object(&mut self, offset: u64) -> Result<()> {
        // Result<PackObjectType> {

        // use GitObjectType::*;
        // use PackObjectType::*;

        let (object_type, object_size) = self.read_type_and_size(offset)?;

        let is_object = match object_type {
            1..=4 => true,
            _ => false,
        };

        let _ = match is_object {
            true => {
                let mut contents = Vec::with_capacity(object_size);
                ZlibDecoder::new(self.file.data.by_ref()).read_to_end(&mut contents)?;
                if contents.len() != object_size {
                    debug!("Contents size and object size aren't the same! Contents: '{}' & Object Size: '{object_size}'", contents.len());
                }
                // assert_eq!(contents.len(), object_size);
                debug!("{:?}", String::from_utf8_lossy(contents.as_slice()));
            },
            false => {
                // todo!()
            },
        };

        return Ok(());
    }

    pub fn run(&mut self) -> Result<()> {
        let hashes: Vec<Hash> = vec![
            Hash(b"\x53\x71\x41\x95\xca\xec\x3f\xdb\xca\xf2\x1d\x4f\x1e\xc5\x19\x11\xfd\x2d\x5c\xb6".to_owned()),
            Hash(b"\x53\x43\x95\x3d\xa4\xcf\x06\x59\xa2\x5f\x63\xd0\x33\x87\xe7\x78\x6e\x89\x0e\xd1".to_owned()),
            Hash(b"\x53\x4e\x71\xdc\x55\xa7\xf4\x90\xdc\xd3\xaa\x53\x4f\x16\x27\x1c\xda\x92\xab\xeb".to_owned()),
            Hash(b"\x53\x53\xbc\xbe\xa8\xa1\x73\x77\x2a\x21\x17\x72\x9f\xe0\x6c\x6b\xf7\x35\xd0\x1f".to_owned()),
        ];

        for &hash in hashes.iter() {
            let offset = self.get_pack_offset(hash).unwrap().unwrap();
            println!("{:?}", self.read_pack_object(offset).unwrap());
        }
        return Ok(());
    }
}
