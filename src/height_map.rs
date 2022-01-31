use bevy::{
    math::Vec3,
    prelude::{Image, Mesh, Quat, Transform},
    render::mesh::Indices,
    render::render_resource::{PrimitiveTopology, TextureFormat},
};
use rand::Rng;

use std::cmp::min;
use std::time::Instant;

pub mod loader;
pub mod rtin;

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
    format: TextureFormat,
}

impl ImageHeightSource {
    pub fn from_grayscale(image: Image) -> ImageHeightSource {
        let height = image.texture_descriptor.size.height as usize;
        let format = image.texture_descriptor.format;
        Self {
            height,
            format,
            image,
        }
    }
}

impl HeightSource for ImageHeightSource {
    #[inline]
    fn sample_height(&self, x: usize, y: usize) -> f32 {
        match self.format {
            TextureFormat::Rgba8UnormSrgb => {
                let offset = (x + (y * self.height)) * 4;
                let srgb = palette::Srgb::from_components((
                    self.image.data[offset] as f64 / 255.0,
                    self.image.data[offset + 1] as f64 / 255.0,
                    self.image.data[offset + 2] as f64 / 255.0,
                ));
                srgb.into_linear().into_components().0 as f32
            }
            TextureFormat::Rgba8Uint => {
                let offset = (x + (y * self.height)) * 4;
                self.image.data[offset] as f32 / 512.0
            }
            TextureFormat::Rgba16Uint => {
                let offset = (x + (y * self.height)) * 8;
                ((self.image.data[offset + 1] as usize) * 256 + self.image.data[offset] as usize)
                    as f32
                    / 131072.0
            }
            _ => panic!("unsupported texture format: {:?}", self.format),
        }
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

    let delta_right = if x >= (height_map.source_size - 1) {
        0.0
    } else {
        center_height - height_map.sample(x + 1, y)
    };

    let delta_top = if y == 0 {
        0.0
    } else {
        center_height - height_map.sample(x, y - 1)
    };

    let delta_bottom = if y >= (height_map.source_size - 1) {
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

use rtin::*;

fn create_mesh<T: HeightSource>(hm: HeightMap<T>) -> Mesh {
    let start = Instant::now();

    // we want {resolution}-1 by {resolution}-1 tiles
    // +---+---+---+
    // |  /|  /|  /|
    // |/  |/  |/  |
    // +---+---+---+
    // map resolution: 2 x 4, grid has 3 tiles, 6 trinangles (w-1) * (h-1) * 2
    let resolution = dbg!(hm.source_size);
    // we want the terrain to occupy size * size units
    let size = hm.target_size;

    let mut positions = vec![[0.0, 0.0, 0.0]; dbg!(resolution * resolution)];
    let mut uvs = vec![[0.0, 0.0]; resolution * resolution];
    let mut normals = vec![[0.0, 0.0, 0.0]; resolution * resolution];
    //let mut indices = Vec::with_capacity(dbg!(resolution * resolution) * 6);

    let res_scale = size as f32 / resolution as f32;
    let half_res = resolution as f32 / 2.0;

    for y in 0..resolution {
        for x in 0..resolution {
            let lx = (x as f32 - half_res) * res_scale;
            let ly = (y as f32 - half_res) * res_scale;

            let height = hm.sample(x, y);
            let offset = x + (y * resolution);
            positions[offset] = [lx, height, ly];
            uvs[offset] = [lx, ly];
            normals[offset] = build_normal(x, y, &hm);
        }
    }


    println!("terrain generation took {:?}", start.elapsed());
    let rtin = RtinMeshBuilder::from_height_map(hm);
    let max_error = 0.002; // TODO: this is a configurable we want to tweak later
    let indices = rtin.get_indices(max_error);
    //TODO refactor this code se we can have a "trivial but slow" implementation, and an RTIN
    // bases "fast" one.
    /*let res_plus1 = resolution + 0;
    for px in 0..(resolution - 1) {
        for py in 0..(resolution - 1) {
            for off in [resolution, 1, 0, resolution, 1 + resolution, 1] {
                indices.push((px + (py * res_plus1) + off) as u32);
            }
        }
    }*/


    println!("terrain generation took {:?}", start.elapsed());
    let indices = Indices::U32(indices);

    let max_triangles = (resolution - 1) * (resolution - 1) * 2;
    println!(
        "{} triangles in total (max {}, {}% saved)",
        indices.len() / 3,
        max_triangles,
        100.0 - (100.0 / max_triangles as f32 * (indices.len() / 3) as f32)
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    println!("{} vertices in total", mesh.count_vertices());

    mesh
}

pub fn mesh_from_image(height_map: Image) -> Mesh {
    let width = height_map.texture_descriptor.size.width;
    let height = height_map.texture_descriptor.size.height;
    let height_source = ImageHeightSource::from_grayscale(height_map);

    let hm = HeightMap::create(
        height_source,
        min(width as usize, height as usize),
        100.0,
    );

    create_mesh(hm)
}
