use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    TargetBin,
};

use crate::error::*;
use crate::graphics;
use crate::resource;

// Steps we need to do:
// - Load resource archive
// - Load in part bitmaps
// - Add part bitmaps to atlas obj
// - Sort out rectangle arrangement, there should be a key of (filename, frame_id)
// - Preallocate a 1024x1024 texture
// - Blit bitmaps to texture based on destination rects
// - There should be a numeric ID for each frame, which can be fetched based on key
// - Display full atlas in mini window

pub struct Atlas<'a> {
    sources: Vec<graphics::TIM3Bitmap<'a>>,
}

impl<'a> Atlas<'a> {
    pub fn new() -> BMResult<Self> {
        Ok(Atlas { sources: vec![] })
    }

    pub fn add_bitmap(&mut self, bitmap: graphics::TIM3Bitmap<'a>) {
        self.sources.push(bitmap);
    }
}

pub fn test_atlas() -> BMResult<()> {
    let mut file = std::fs::File::open("RESOURCE.MAP")?;
    let mut resource_map = resource::TIM3ResourceMap::new(&mut file)?;

    let mut resource_001 = std::fs::File::open("RESOURCE.001")?;
    resource_map.add_archive("RESOURCE.001", &mut resource_001)?;
    let mut resource_002 = std::fs::File::open("RESOURCE.002")?;
    resource_map.add_archive("RESOURCE.002", &mut resource_002)?;

    let mut palette_data = resource_map.get("TIM.PAL").ok_or(BMError::LoadingError {
        msg: "Missing palette".into(),
    })?;
    let palette =
        graphics::TIM3Palette::new("TIM.PAL", &mut std::io::Cursor::new(palette_data))?.get_palette()?;

    let mut atlas = Atlas::new()?;

    // Iterate through all part bitmaps
    let key_iter = resource_map
        .keys()
        .filter(|k| k.starts_with("PART") && k.ends_with(".BMP"));

    for k in key_iter {
        let bitmap_data = resource_map.get(k).ok_or(BMError::LoadingError {
            msg: "Missing bitmap".into(),
        })?;
        let mut bitmap = graphics::TIM3Bitmap::new(k, &mut std::io::Cursor::new(bitmap_data))?;
        bitmap.set_palette(&palette)?;
        atlas.add_bitmap(bitmap);
    }

    Ok(())
}
