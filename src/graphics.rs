use std::io;
use std::io::prelude::*;

use byteorder::{BigEndian as BE, LittleEndian as LE, ReadBytesExt};
use bytes::{Bytes, BytesMut};
use sdl2::{pixels, surface};

use crate::error::*;

pub struct TIM3Palette {
    pub name: String,
    pub colours: Vec<pixels::Color>,
}

impl TIM3Palette {
    pub fn new<R: std::io::Read + std::io::Seek>(name: &str, stream: &mut R) -> BMResult<Self> {
        // Check the magic number for the file; should be RIFF
        let magic = stream.read_u32::<BE>()?;
        if magic != 0x52494646 {
            return Err(BMError::LoadingError {
                msg: "Expected RIFF for palette".into(),
            });
        }
        stream.read_u32::<LE>()?; // RIFF size

        // Check the magic type ID; should be 'PAL data'
        let magic = stream.read_u64::<BE>()?;
        if magic != 0x50414c2064617461 {
            return Err(BMError::LoadingError {
                msg: "Expected 'PAL data' for palette".into(),
            });
        }

        let inner_size = stream.read_u32::<LE>()? as usize;
        let mut inner_payload = BytesMut::with_capacity(inner_size);
        stream.read_exact(&mut inner_payload)?;
        let inner_cursor = std::io::Cursor::new(&inner_payload);
        let version = stream.read_u16::<LE>()?;
        if version != 0x300 {
            return Err(BMError::LoadingError {
                msg: "Expected version 300 RIFF palette".into(),
            });
        }
        let pal_count = stream.read_u16::<LE>()?;

        let mut result = TIM3Palette {
            name: name.into(),
            colours: vec![],
        };

        for i in 0..pal_count {
            let r = stream.read_u8()?;
            let g = stream.read_u8()?;
            let b = stream.read_u8()?;
            let flags = stream.read_u8()?;
            result.colours.push(pixels::Color::RGB(r, g, b));
        }

        Ok(result)
    }

    pub fn get_palette(&self) -> BMResult<pixels::Palette> {
        Ok(pixels::Palette::with_colors(&self.colours)
            .or_else(|err| Err(BMError::LoadingError { msg: err.into() }))?)
    }
}

pub struct TIM3Bitmap<'a> {
    pub name: String,
    pub frames: Vec<surface::Surface<'a>>,
}

impl<'a> TIM3Bitmap<'a> {
    pub fn new<R: std::io::Read + std::io::Seek>(name: &str, stream: &mut R) -> BMResult<Self> {
        // Check the magic number for the file; should be BMP:
        let magic = stream.read_u32::<BE>()?;
        if magic != 0x424d503a {
            return Err(BMError::LoadingError {
                msg: "Invalid magic ID for TIM3Bitmap".into(),
            });
        }
        // Get the payload size and read in the payload
        let size = stream.read_u32::<LE>()?;
        let mut payload = BytesMut::with_capacity(size as usize);
        stream.read_exact(&mut payload)?;

        // Create a cursor to track the frame offset list
        let mut cursor = std::io::Cursor::new(&payload);
        cursor.read_u16::<LE>()?; // unknown
        let frame_count = cursor.read_u16::<LE>()?;

        let mut result = TIM3Bitmap {
            name: name.into(),
            frames: vec![],
        };

        for _ in 0..frame_count {
            // Read a frame offset from the table
            let frame_offset = cursor.read_u32::<LE>()?;
            // Make a new cursor for the frame and jump it to the offset
            let mut frame_cursor = std::io::Cursor::new(&payload);
            frame_cursor.seek(std::io::SeekFrom::Start(frame_offset as u64 - 8))?;
            // Pull the image dimensions from the frame cursor
            let width = frame_cursor.read_u16::<LE>()? as u32;
            let height = frame_cursor.read_u16::<LE>()? as u32;
            frame_cursor.read_u8()?; // unknown
            let frame_size = frame_cursor.read_u32::<LE>()? as usize;

            // Pull the raw 8-bit image data,
            // create a local surface handle for it
            let mut frame_data: Vec<u8> = vec![0; frame_size];
            frame_cursor.read_exact(&mut frame_data)?;
            let frame_surface = surface::Surface::from_data(
                &mut frame_data,
                width,
                height,
                width,
                pixels::PixelFormatEnum::Index8,
            )
            .or_else(|err| Err(BMError::LoadingError { msg: err.into() }))?;

            // Make a copy of the surface with .convert_format(),
            // store it in the frames list
            result.frames.push(
                frame_surface
                    .convert_format(frame_surface.pixel_format_enum())
                    .or_else(|err| Err(BMError::LoadingError { msg: err.into() }))?,
            );
        }

        Ok(result)
    }

    pub fn set_palette(&mut self, palette: &pixels::Palette) -> BMResult<()> {
        for frame in self.frames.iter_mut() {
            frame
                .set_palette(&palette)
                .or_else(|err| Err(BMError::LoadingError { msg: err.into() }))?;
        }
        Ok(())
    }
}
