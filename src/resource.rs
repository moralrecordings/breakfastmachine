use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::str;

use byteorder::{LittleEndian as LE, ReadBytesExt};
use bytes::{Bytes, BytesMut};

use crate::error::*;

pub struct TIM3ResourceMap {
    offsets: HashMap<String, Vec<u32>>,
    files: HashMap<String, Bytes>,
}

impl TIM3ResourceMap {
    pub fn new<R: std::io::Read + std::io::Seek>(stream: &mut R) -> BMResult<Self> {
        let magic = stream.read_u32::<LE>()?;
        if magic != 0x07040100 {
            return Err(BMError::LoadingError {
                msg: "Invalid magic ID for TIM3ResourceMap".into(),
            });
        }

        let resource_count = stream.read_u16::<LE>()?;
        let mut offsets = HashMap::new();
        let files = HashMap::new();
        for _ in 0..resource_count {
            let mut resource_file_name = [0; 13];
            stream.read_exact(&mut resource_file_name)?;
            let key = str::from_utf8(&resource_file_name)?
                .trim_end_matches('\0')
                .to_uppercase()
                .to_string();

            let resource_file_count = stream.read_u16::<LE>()?;
            let mut file_offsets = vec![];
            for _ in 0..resource_file_count {
                stream.read_u32::<LE>()?; // file name hash
                file_offsets.push(stream.read_u32::<LE>()?);
            }
            offsets.insert(key, file_offsets);
        }
        Ok(TIM3ResourceMap { offsets, files })
    }

    pub fn add_archive<R: std::io::Read + std::io::Seek>(
        &mut self,
        name: &str,
        stream: &mut R,
    ) -> BMResult<()> {
        let name_string = name.to_string().to_uppercase();
        let file_offsets = self
            .offsets
            .get(&name_string)
            .ok_or(BMError::LoadingError {
                msg: format!("No record in resource map found for {}", name_string),
            })?;

        for offset in file_offsets.iter() {
            stream.seek(std::io::SeekFrom::Start(*offset as u64))?;
            let mut file_name = [0; 13];
            stream.read_exact(&mut file_name)?;
            let key = str::from_utf8(&file_name)?
                .trim_end_matches('\0')
                .to_string();

            let file_size = stream.read_u16::<LE>()?;
            let mut file_data = BytesMut::with_capacity(file_size.into());
            stream.read_exact(file_data.as_mut())?;

            self.files.insert(key, file_data.into());
        }

        Ok(())
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<String, Bytes> {
        self.files.keys()
    }

    pub fn get(&self, key: &str) -> Option<&Bytes> {
        self.files.get(key)
    }
}
