use std::ffi::OsStr;

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ImageFormat {
    Unknown,

    /// An Image in PNG Format
    Png,

    /// An Image in JPEG Format
    Jpeg,

    /// An Image in GIF Format
    Gif,

    /// An Image in WEBP Format
    WebP,

    /// An Image in general PNM Format
    Pnm,

    /// An Image in TIFF Format
    Tiff,

    /// An Image in TGA Format
    Tga,

    /// An Image in DDS Format
    Dds,

    /// An Image in BMP Format
    Bmp,

    /// An Image in ICO Format
    Ico,

    /// An Image in Radiance HDR Format
    Hdr,

    /// An Image in OpenEXR Format
    OpenExr,

    /// An Image in farbfeld Format
    Farbfeld,

    /// An Image in AVIF Format
    Avif,

    /// An Image in QOI Format
    Qoi,

    /// An Image in SVG Format
    Svg
}

impl ImageFormat {
    #[inline]
    pub fn from_extension<S>(ext: S) -> ImageFormat
    where
        S: AsRef<str>,
    {
        let ext = ext.as_ref().to_ascii_lowercase();

        match ext.as_str() {
            "avif" => ImageFormat::Avif,
            "jpg" | "jpeg" | "jfif" => ImageFormat::Jpeg,
            "png" | "apng" => ImageFormat::Png,
            "svg" => ImageFormat::Svg,
            "gif" => ImageFormat::Gif,
            "webp" => ImageFormat::WebP,
            "tif" | "tiff" => ImageFormat::Tiff,
            "tga" => ImageFormat::Tga,
            "dds" => ImageFormat::Dds,
            "bmp" => ImageFormat::Bmp,
            "ico" => ImageFormat::Ico,
            "hdr" => ImageFormat::Hdr,
            "exr" => ImageFormat::OpenExr,
            "pbm" | "pam" | "ppm" | "pgm" | "pnm" => ImageFormat::Pnm,
            "ff" => ImageFormat::Farbfeld,
            "qoi" => ImageFormat::Qoi,
            _ => ImageFormat::Unknown,
        }
    }
}