use crate::height_map;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::{FromWorld, Image, World},
    render::renderer::RenderDevice,
    render::texture::{CompressedImageFormats, ImageType},
};

pub struct HeightmapMeshLoader {
    supported_compressed_formats: CompressedImageFormats,
}

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

            let dyn_img = Image::from_buffer(
                bytes,
                ImageType::Extension(ext),
                self.supported_compressed_formats,
                // not sure about this one as of yet. can we know?
                // guess we could let use `image` to parse the image and find out, but then we
                // might not want to use `from_buffer` at all, because that does that, too?
                true,
            )
            .unwrap();
            let mesh = height_map::mesh_from_image(dyn_img);
            load_context.set_default_asset(LoadedAsset::new(mesh));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["hm.png", "hm.jpg"]
    }
}

impl FromWorld for HeightmapMeshLoader {
    fn from_world(world: &mut World) -> Self {
        let supported_compressed_formats = match world.get_resource::<RenderDevice>() {
            Some(render_device) => CompressedImageFormats::from_features(render_device.features()),

            None => CompressedImageFormats::all(),
        };
        Self {
            supported_compressed_formats,
        }
    }
}
