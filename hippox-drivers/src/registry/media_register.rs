//! Media processing drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Media;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "media", feature = "all"))]
    {
        use crate::{
            BarcodeGenerateDriver, BarcodeParseDriver, ImageBatchConvertDriver, ImageCompressDriver, ImageConvertDriver, ImageCropDriver, ImageExifDriver, ImageFilterDriver, ImageInfoDriver, ImageResizeDriver, ImageRotateDriver, ImageStitchDriver, ImageWatermarkDriver, OcrDriver, QrCodeGenerateDriver, QrCodeParseDriver, ScreenshotDriver
        };
        // Image drivers
        map.insert("image_resize".to_string(), Arc::new(ImageResizeDriver));
        map.insert("image_convert".to_string(), Arc::new(ImageConvertDriver));
        map.insert("image_info".to_string(), Arc::new(ImageInfoDriver));
        map.insert("image_rotate".to_string(), Arc::new(ImageRotateDriver));
        map.insert("image_crop".to_string(), Arc::new(ImageCropDriver));
        map.insert("image_compress".to_string(), Arc::new(ImageCompressDriver));
        map.insert("image_filter".to_string(), Arc::new(ImageFilterDriver));
        map.insert("image_watermark".to_string(), Arc::new(ImageWatermarkDriver));
        map.insert("image_stitch".to_string(), Arc::new(ImageStitchDriver));
        map.insert("image_exif".to_string(), Arc::new(ImageExifDriver));
        map.insert(
            "image_batch_convert".to_string(),
            Arc::new(ImageBatchConvertDriver),
        );
        // QR Code drivers
        map.insert("qrcode_generate".to_string(), Arc::new(QrCodeGenerateDriver));
        map.insert("qrcode_parse".to_string(), Arc::new(QrCodeParseDriver));
        // Barcode drivers
        map.insert(
            "barcode_generate".to_string(),
            Arc::new(BarcodeGenerateDriver),
        );
        map.insert("barcode_parse".to_string(), Arc::new(BarcodeParseDriver));
        // OCR skill
        map.insert("ocr".to_string(), Arc::new(OcrDriver));
        // Screenshot skill
        map.insert("screenshot".to_string(), Arc::new(ScreenshotDriver));
    }
}
