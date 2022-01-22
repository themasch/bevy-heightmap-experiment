use bevy::prelude::{Image, Mesh, Quat, Transform};
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use rand::Rng;

use bevy::math::Vec3;
use std::cmp::min;
use std::time::Instant;

pub mod loader;

pub trait HeightSource {
    fn sample_height(&self, x: usize, y: usize) -> f32;
}

pub struct ThreadLocalRngHeightSource;

impl ThreadLocalRngHeightSource {
    #[allow(dead_code)]
    fn new() -> Self {
        Self
    }
}

impl HeightSource for ThreadLocalRngHeightSource {
    fn sample_height(&self, _: usize, _: usize) -> f32 {
        rand::thread_rng().gen_range(-2..2) as f32 / 100.0
    }
}

pub struct ImageHeightSource {
    image: Image,
    height: usize,
    bytes_per_pixel: usize,
}

impl ImageHeightSource {
    pub fn from_grayscale(image: Image) -> ImageHeightSource {
        let width = image.texture_descriptor.size.width as usize;
        let height = image.texture_descriptor.size.height as usize;
        Self {
            height,
            bytes_per_pixel: image.data.len() / (width * height),
            image,
        }
    }
}

impl HeightSource for ImageHeightSource {
    #[inline]
    fn sample_height(&self, x: usize, y: usize) -> f32 {
        let offset = (x + (y * self.height)) * self.bytes_per_pixel;

        self.image.data[offset] as f32 / 512.0
    }
}

pub struct HeightMap<H>
    where
        H: HeightSource,
{
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

    fn sample(&self, x: usize, y: usize) -> f32 {
        debug_assert!(x <= self.source_size);
        debug_assert!(y <= self.source_size);
        <H as HeightSource>::sample_height(&self.height_source, x, y)
    }
}

fn build_normal<T: HeightSource>(x: usize, y: usize, height_map: &HeightMap<T>) -> [f32; 3] {
    let center_height = height_map.sample(x, y);

    let delta_left = if x == 0 {
        0.0
    } else {
        center_height - height_map.sample(x - 1, y)
    };

    let delta_right = if x == height_map.source_size {
        0.0
    } else {
        center_height - height_map.sample(x + 1, y)
    };

    let delta_top = if y == 0 {
        0.0
    } else {
        center_height - height_map.sample(x, y - 1)
    };

    let delta_bottom = if y == height_map.source_size {
        0.0
    } else {
        center_height - height_map.sample(x, y + 1)
    };

    // quick path for flat terrain
    if delta_bottom == 0.0 && delta_top == 0.0 && delta_left == 0.0 && delta_right == 0.0 {
        return [0.0, 1.0, 0.0];
    }

    let x_angel = ((delta_left - delta_right) / 2.0) * 45.0;
    let y_angel = ((delta_top - delta_bottom) / 2.0) * 45.0;

    let mut transform = Transform::identity();
    transform.rotate(Quat::from_rotation_x(x_angel));
    transform.rotate(Quat::from_rotation_y(y_angel));

    let normal = transform * Vec3::new(0.0, 1.0, 0.0);

    normal.to_array()
}

fn create_mesh<T: HeightSource>(hm: HeightMap<T>) -> Mesh {
    let start = Instant::now();

    // we want {resolution} by {resolution} tiles
    let resolution = hm.source_size;
    // we want the terrain to occupy size * size units
    let size = hm.target_size;

    let mut positions = vec![[0.0, 0.0, 0.0]; (resolution + 1) * (resolution + 1)];
    let mut uvs = vec![[0.0, 0.0]; (resolution + 1) * (resolution + 1)];
    let mut normals = vec![[0.0, 0.0, 0.0]; (resolution + 1) * (resolution + 1)];
    let mut indices = Vec::with_capacity(resolution * resolution * 6);

    let res_scale = size as f32 / resolution as f32;
    let half_res = resolution as f32 / 2.0;

    for x in 0..=resolution {
        for y in 0..=resolution {
            let lx = (x as f32 - half_res) * res_scale;
            let ly = (y as f32 - half_res) * res_scale;

            let height = hm.sample(x, y);
            let offset = y + (x * (resolution + 1));
            positions[offset] = [lx, height, ly];
            uvs[offset] = [lx, ly];
            normals[offset] = build_normal(x, y, &hm);
        }
    }

    println!("terrain generation took {:?}", start.elapsed());
    let res_plus1 = resolution + 1;
    for py in 0..resolution {
        for px in 0..resolution {
            for off in [0, 1, res_plus1, 1, 1 + res_plus1, res_plus1] {
                indices.push((px + (py * res_plus1) + off) as u32);
            }
        }
    }

    println!("{} triangles in total", positions.len());

    println!("terrain generation took {:?}", start.elapsed());
    let indices = Indices::U32(indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}

pub fn mesh_from_image(height_map: Image) -> Mesh {
    let width = height_map.texture_descriptor.size.width;
    let height = height_map.texture_descriptor.size.height;
    let height_source = ImageHeightSource::from_grayscale(height_map);

    let hm = HeightMap::create(
        height_source,
        min(width as usize, height as usize) - 10,
        10.0,
    );

    create_mesh(hm)
}
