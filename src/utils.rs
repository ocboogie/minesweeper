use egui::{Image, ImageSource, TextureOptions};

pub fn load_image(src: ImageSource) -> Image<'_> {
    Image::new(src)
        .fit_to_original_size(1.0)
        .texture_options(TextureOptions::NEAREST)
}
