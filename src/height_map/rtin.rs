/// TODO: understand this code
/// this is mostly copied/transliterated from [a javascript example by Vladimir Agafonkin][0]
///
/// I have read [the paper][1], but still do not quite understand whats going on. But it just works.
/// And it works great! While it adds some time for setting up the mesh over the "initial stupid"
/// implementation (aka "just render all triangle lel"), it reduces the number of drawn triangles
/// by a lot, even without LOD based on the camera position
///
/// [0]: https://observablehq.com/@mourner/martin-real-time-rtin-terrain-mesh
/// [1]: https://www.cs.ubc.ca/~will/papers/rtin.pdf
use crate::height_map::{HeightMap, HeightSource};

#[derive(Debug, Clone, Copy)]
struct UXY {
    x: usize,
    y: usize,
}

impl UXY {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn middle_of(a: &UXY, b: &UXY) -> Self {
        Self {
            x: (a.x + b.x) / 2,
            y: (a.y + b.y) / 2,
        }
    }

    fn as_offset(&self, grid_size: usize) -> usize {
        self.y * grid_size + self.x
    }
}

struct ErrorMap {
    data: Vec<f32>,
    grid_size: usize,
}

impl ErrorMap {
    pub(crate) fn get_error(&self, pos: &UXY) -> f32 {
        self.data[pos.as_offset(self.grid_size)]
    }
}

impl ErrorMap {
    fn from_height_map<T: HeightSource>(hm: &HeightMap<T>) -> ErrorMap {
        let grid_size = hm.source_size;
        let tile_size = hm.source_size - 1;
        let mut errors = vec![0.0; grid_size * grid_size];

        let number_of_smallest_triangles = tile_size * tile_size;
        let number_of_all_triangles = number_of_smallest_triangles * 2 - 2;
        let last_level_index = number_of_all_triangles - number_of_smallest_triangles;

        for idx in (0..=number_of_all_triangles).rev() {
            let mut id = idx + 2;
            let mut a = UXY::new(0, 0);
            let mut b = UXY::new(0, 0);
            let mut c = UXY::new(0, 0);

            if id & 1 == 1 {
                b.x = tile_size;
                b.y = tile_size;
                c.x = tile_size;
            } else {
                a.x = tile_size;
                a.y = tile_size;
                c.y = tile_size;
            }

            while id / 2 > 1 {
                id = id / 2;

                let m = UXY::new((a.x + b.x) / 2, (a.y + b.y) / 2);

                if id & 1 == 1 {
                    b = a;
                    a = c;
                } else {
                    a = b;
                    b = c;
                }

                c = m;
            }

            let center = UXY::middle_of(&a, &b);

            let interpolated_height = (hm.sample(a.x, a.y) + hm.sample(b.x, b.y)) / 2.0;
            let center_height = hm.sample(center.x, center.y);
            let center_error = (interpolated_height - center_height).abs();

            let new_error = if idx >= last_level_index {
                center_error
            } else {
                let left_child = UXY::middle_of(&a, &c);
                let left_child_error = errors[left_child.as_offset(grid_size)];
                let right_child = UXY::middle_of(&b, &c);
                let right_child_error = errors[right_child.as_offset(grid_size)];
                f32::max(
                    f32::max(errors[center.as_offset(grid_size)], center_error),
                    f32::max(left_child_error, right_child_error),
                )
            };

            errors[center.as_offset(grid_size)] = new_error;
        }

        Self {
            data: errors,
            grid_size,
        }
    }
}

pub struct RtinMeshBuilder<T: HeightSource> {
    height_map: HeightMap<T>,
    error_map: ErrorMap,
}

impl<T: HeightSource> RtinMeshBuilder<T> {
    pub fn from_height_map(height_map: HeightMap<T>) -> Self {
        let error_map = ErrorMap::from_height_map(&height_map);
        Self {
            height_map,
            error_map,
        }
    }

    pub fn get_indices(&self, max_error: f32) -> Vec<u32> {
        let builder = IndexBuilder::create(&self.error_map, self.height_map.source_size, max_error);

        builder.process_root()
    }
}

struct IndexBuilder<'e> {
    current_index: usize,
    indices: Vec<u32>,
    max_error: f32,
    grid_size: usize,
    error_map: &'e ErrorMap,
}

impl<'e> IndexBuilder<'e> {
    fn create(error_map: &'e ErrorMap, grid_size: usize, max_error: f32) -> Self {
        debug_assert!(max_error >= 0.0);

        Self {
            current_index: 0,
            // there are at most {grid_size} -1 * {grid_size - 1} * 2 triangles, each needing 3 idx
            indices: vec![0u32; (grid_size - 1) * (grid_size - 1) * 2 * 3],
            max_error,
            grid_size,
            error_map,
        }
    }

    fn process_root(mut self) -> Vec<u32> {
        self.process_triangle(
            UXY::new(0, 0),
            UXY::new(self.grid_size - 1, self.grid_size - 1),
            UXY::new(self.grid_size - 1, 0),
        );
        self.process_triangle(
            UXY::new(self.grid_size - 1, self.grid_size - 1),
            UXY::new(0, 0),
            UXY::new(0, self.grid_size - 1),
        );

        self.indices.truncate(self.current_index);
        self.indices
    }

    fn process_triangle(&mut self, a: UXY, b: UXY, c: UXY) {
        let m = UXY::middle_of(&a, &b);
        if (a.x as isize - c.x as isize).abs() + (a.y as isize - c.y as isize).abs() > 1
            && self.error_map.get_error(&m) > self.max_error
        {
            self.process_triangle(c, a, m);
            self.process_triangle(b, c, m);
        } else {
            self.push_index(a);
            self.push_index(b);
            self.push_index(c);
        }
    }

    fn push_index(&mut self, pos: UXY) {
        let index = pos.as_offset(self.grid_size) as u32;
        self.indices[self.current_index] = index;
        self.current_index += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bevy::winit::winit_runner;

    struct TestHeightSource<const W: usize> {
        data: Vec<f32>,
    }

    impl<const W: usize> HeightSource for TestHeightSource<W> {
        fn sample_height(&self, x: usize, y: usize) -> f32 {
            self.data[y * W + x]
        }
    }

    #[test]
    fn test_error_map_is_build_correctly_small_dataset() {
        let source: TestHeightSource<3> = TestHeightSource {
            data: vec![0.0, 1.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 0.0],
        };
        let hm = HeightMap::create(source, 3, 1.0);

        let expected_result = vec![0.0, 0.5, 0.0, 2.0, 2.5, 2.5, 0.0, 0.0, 0.0];

        let error_map = ErrorMap::from_height_map(&hm);

        assert_eq!(expected_result, error_map.data);
    }

    #[test]
    fn test_indices_are_build_correctly_small_dataset() {
        let source: TestHeightSource<3> = TestHeightSource {
            data: vec![0.0, 1.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 0.0],
        };
        let hm = HeightMap::create(source, 3, 1.0);

        let expected_result = vec![
            4, 2, 1, 0, 4, 1, 4, 8, 5, 2, 4, 5, 6, 8, 4, 4, 0, 3, 6, 4, 3,
        ];

        let rtin = RtinMeshBuilder::from_height_map(hm);

        let indices = rtin.get_indices(0.0);
        assert_eq!(expected_result, indices);
    }
}
