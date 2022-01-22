use bevy::prelude::Image;
use bevy::render::texture::ImageType;
use venture::height_map;

fn main() {
    let bytes = include_bytes!("../../assets/Sc2wB.hm.jpg");
    let dyn_img = Image::from_buffer(bytes, ImageType::MimeType("image/jpeg")).unwrap();
    height_map::mesh_from_image(dyn_img);
}
