use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::{Image, Mesh, Quat, Transform};
use rand::prelude::ThreadRng;
use rand::Rng;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::texture::ImageType;

use std::time::Instant;
use std::cmp::min;
use bevy::math::Vec3;

pub mod loader;

pub trait HeightSource {
    fn sample_height(&mut self, x: usize, y: usize) -> f32;
}

pub struct ThreadLocalRngHeightSource {
    rng: ThreadRng,
}

impl ThreadLocalRngHeightSource {
    fn new() -> Self {
        Self { rng: rand::thread_rng() }
    }
}

impl HeightSource for ThreadLocalRngHeightSource {
    fn sample_height(&mut self, _: usize, _: usize) -> f32 {
        self.rng.gen_range(-2..2) as f32 / 100.0
    }
}

pub struct ImageHeightSource {
    image: Image,
}

impl HeightSource for ImageHeightSource {
    fn sample_height(&mut self, x: usize, y: usize) -> f32 {
        let width = self.image.texture_descriptor.size.width as usize;
        let height = self.image.texture_descriptor.size.height as usize;
        let bytes_per_pixel = self.image.data.len() / (width * height);
        let offset = (x + (y * height)) * bytes_per_pixel;

        debug_assert!(x < width);
        debug_assert!(y < height);

        (self.image.data[offset] as f32 / 255.0) * 0.25
    }
}

pub struct HeightMap<H> where H: HeightSource {
    height_source: H,
    source_size: usize,
    target_size: f32,
}

impl<H: HeightSource> HeightMap<H> {
    pub fn create(height_source: H, source_size: usize, target_size: f32) -> HeightMap<H> {
        Self {
            height_source,
            source_size,
            target_size,
        }
    }

    fn sample(&mut self, x: usize, y: usize) -> f32 {
        debug_assert!(x <= self.source_size);
        debug_assert!(y <= self.source_size);
        <H as HeightSource>::sample_height(&mut self.height_source, x, y)
    }
}

fn build_normal<T: HeightSource>(x: usize, y: usize, height_map: &mut HeightMap<T>) -> [f32; 3] {
    let center_height = height_map.sample(x, y);

    let delta_left = if x == 0 { 0.0 } else {
        center_height - height_map.sample(x - 1, y)
    };

    let delta_right = if x == height_map.source_size { 0.0 } else {
        center_height - height_map.sample(x + 1, y)
    };

    let delta_top = if y == 0 { 0.0 } else {
        center_height - height_map.sample(x, y - 1)
    };

    let delta_bottom = if y == height_map.source_size { 0.0 } else {
        center_height - height_map.sample(x, y + 1)
    };

    let x_angel = ((delta_left - delta_right) / 2.0) * 45.0;
    let y_angel = ((delta_top - delta_bottom) / 2.0) * 45.0;

    let mut transform = Transform::identity();
    transform.rotate(Quat::from_rotation_x(x_angel));
    transform.rotate(Quat::from_rotation_y(y_angel));

    let normal = transform * Vec3::new(0.0, 1.0, 0.0);

    normal.to_array()
}

fn create_mesh<T: HeightSource>(mut hm: HeightMap<T>) -> Mesh {
    let start = Instant::now();

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // we want {resolution} by {resolution} tiles
    let resolution = hm.source_size;
    // we want the terrain to occupy size * size units
    let size = hm.target_size;

    let res_scale = size as f32 / resolution as f32;

    for x in 0..=resolution {
        for y in 0..=resolution {
            let lx = (x as f32 - (resolution as f32 / 2.0)) * res_scale;
            let ly = (y as f32 - (resolution as f32 / 2.0)) * res_scale;

            let height = hm.sample(x, y);
            positions.push([lx, height, ly]);
            uvs.push([lx, ly]);
            normals.push(build_normal(x, y, &mut hm));
        }
    }

    let res_plus1 = resolution + 1;
    for py in 0..resolution {
        for px in 0..resolution {
            for off in [0, 1, res_plus1, 1, 1 + res_plus1, res_plus1] {
                indices.push((px + (py * res_plus1) + off) as u32);
            }
        }
    }

    let indices = Indices::U32(indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    println!("terrain generation took {:?}", start.elapsed());

    mesh
}

fn mesh_from_image(height_map: Image) -> Mesh {
    let width = height_map.texture_descriptor.size.width;
    let height = height_map.texture_descriptor.size.height;
    let height_source = ImageHeightSource { image: height_map };

    let hm = HeightMap::create(height_source, min(width as usize, height as usize) - 10, 5.0);

    create_mesh(hm)
}
