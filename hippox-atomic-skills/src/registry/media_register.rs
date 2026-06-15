//! Media processing skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Media;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "media", feature = "all"))]
    {
        use crate::skills::image::*;
        
        map.insert("image_resize".to_string(), Arc::new(ImageResizeSkill));
        map.insert("image_convert".to_string(), Arc::new(ImageConvertSkill));
        map.insert("image_info".to_string(), Arc::new(ImageInfoSkill));
        map.insert("image_rotate".to_string(), Arc::new(ImageRotateSkill));
        map.insert("image_crop".to_string(), Arc::new(ImageCropSkill));
        map.insert("image_compress".to_string(), Arc::new(ImageCompressSkill));
    }
}