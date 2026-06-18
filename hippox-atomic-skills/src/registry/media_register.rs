//! Media processing skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Media;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "media", feature = "all"))]
    {
        use crate::{
            BarcodeGenerateSkill, BarcodeParseSkill, ImageBatchConvertSkill, ImageCompressSkill, ImageConvertSkill, ImageCropSkill, ImageExifSkill, ImageFilterSkill, ImageInfoSkill, ImageResizeSkill, ImageRotateSkill, ImageStitchSkill, ImageWatermarkSkill, OcrSkill, QrCodeGenerateSkill, QrCodeParseSkill, ScreenshotSkill
        };
        // Image skills
        map.insert("image_resize".to_string(), Arc::new(ImageResizeSkill));
        map.insert("image_convert".to_string(), Arc::new(ImageConvertSkill));
        map.insert("image_info".to_string(), Arc::new(ImageInfoSkill));
        map.insert("image_rotate".to_string(), Arc::new(ImageRotateSkill));
        map.insert("image_crop".to_string(), Arc::new(ImageCropSkill));
        map.insert("image_compress".to_string(), Arc::new(ImageCompressSkill));
        map.insert("image_filter".to_string(), Arc::new(ImageFilterSkill));
        map.insert("image_watermark".to_string(), Arc::new(ImageWatermarkSkill));
        map.insert("image_stitch".to_string(), Arc::new(ImageStitchSkill));
        map.insert("image_exif".to_string(), Arc::new(ImageExifSkill));
        map.insert(
            "image_batch_convert".to_string(),
            Arc::new(ImageBatchConvertSkill),
        );
        // QR Code skills
        map.insert("qrcode_generate".to_string(), Arc::new(QrCodeGenerateSkill));
        map.insert("qrcode_parse".to_string(), Arc::new(QrCodeParseSkill));
        // Barcode skills
        map.insert(
            "barcode_generate".to_string(),
            Arc::new(BarcodeGenerateSkill),
        );
        map.insert("barcode_parse".to_string(), Arc::new(BarcodeParseSkill));
        // OCR skill
        map.insert("ocr".to_string(), Arc::new(OcrSkill));
        // Screenshot skill
        map.insert("screenshot".to_string(), Arc::new(ScreenshotSkill));
    }
}
