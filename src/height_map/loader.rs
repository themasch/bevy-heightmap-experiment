use crate::height_map;
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::Image;
use bevy::render::texture::ImageType;

pub struct HeightmapMeshLoader;

impl AssetLoader for HeightmapMeshLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        println!("loading height map");
        Box::pin(async move {
            // use the file extension for the image type
            let ext = load_context.path().extension().unwrap().to_str().unwrap();

            let dyn_img = Image::from_buffer(bytes, ImageType::Extension(ext)).unwrap();
            let mesh = height_map::mesh_from_image(dyn_img);
            load_context.set_default_asset(LoadedAsset::new(mesh));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["hm.png", "hm.jpg"]
    }
}
