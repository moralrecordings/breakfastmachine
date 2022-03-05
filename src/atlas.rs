
use crate::error::*;
use crate::resource;

// Steps we need to do:
// - Load resource archive
// - Load in part frames
// - Add part frame rects to atlas
// - Preallocate a 1024x1024 texture
// - Add bitmaps to texture based on atlas
// - Display full atlas in mini window

pub fn test_atlas() -> BMResult<()>  {
    let mut file = std::fs::File::open("RESOURCE.MAP")?;
    let mut resource_map = resource::ResourceMap::new(&mut file)?;
  
    let mut resource_001 = std::fs::File::open("RESOURCE.001")?;
    resource_map.add_archive("RESOURCE.001", &mut resource_001)?;
    let mut resource_002 = std::fs::File::open("RESOURCE.002")?;
    resource_map.add_archive("RESOURCE.002", &mut resource_002)?;




    Ok(())
}
