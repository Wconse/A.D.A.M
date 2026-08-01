//! Deterministic organic province geometry for the strategic map.
//!
//! The strategic map is a perturbed hexagonal lattice. Every lattice corner is
//! displaced by a hash-derived offset keyed on the corner itself, so all cells
//! touching a corner agree on its position: provinces tile the plane without
//! gaps or overlaps while looking hand-drawn rather than tiled.
//!
//! The module owns no simulation rules. It receives the authoritative region
//! order with country ownership and derives stable geometry from a fixed seed,
//! so the same world always produces the same map.

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;

/// Lattice radius of one province before corner perturbation.
pub const HEX_RADIUS: f32 = 34.0;
const COLUMN_RANGE: i32 = 14;
const ROW_RANGE: i32 = 9;
const CORNER_JITTER: f32 = 0.30;
const KEY_SCALE: f32 = 4.0;
const SEA_MARGIN: f32 = 0.93;
const SQRT_3: f32 = 1.732_050_8;

/// Quantised lattice corner identity shared by every touching province.
pub type CornerKey = (i64, i64);

/// Axial neighbour offsets paired with the two local corners they share.
const DIRECTIONS: [((i32, i32), (usize, usize)); 6] = [
    ((1, 0), (0, 1)),
    ((0, 1), (1, 2)),
    ((-1, 1), (2, 3)),
    ((-1, 0), (3, 4)),
    ((0, -1), (4, 5)),
    ((1, -1), (5, 0)),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellKind {
    Sea,
    Land { region: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Relief {
    Plain,
    Forest,
    Hills,
    Mountains,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BorderClass {
    Province,
    Coast,
    Region,
    Country,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MapCell {
    pub axial: (i32, i32),
    pub center: Vec2,
    pub polygon: [Vec2; 6],
    pub kind: CellKind,
    pub tint: f32,
    pub relief: Relief,
}

impl MapCell {
    #[must_use]
    pub const fn region(&self) -> Option<usize> {
        match self.kind {
            CellKind::Land { region } => Some(region),
            CellKind::Sea => None,
        }
    }

    #[must_use]
    pub fn contains(&self, point: Vec2) -> bool {
        point_in_polygon(point, &self.polygon)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BorderSegment {
    pub from: Vec2,
    pub to: Vec2,
    pub class: BorderClass,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StrategicMap {
    pub cells: Vec<MapCell>,
    pub borders: Vec<BorderSegment>,
    pub region_centroids: Vec<Vec2>,
    pub region_capitals: Vec<Vec2>,
    pub region_cell_counts: Vec<usize>,
    pub half_extent: Vec2,
}

impl StrategicMap {
    /// Builds the province blockout for the authoritative region list.
    ///
    /// `region_countries` is indexed by region order and holds the owning
    /// country identifier of that region.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn generate(region_countries: &[u32], seed: u64) -> Self {
        let axials = lattice_axials();
        let centers: Vec<Vec2> = axials.iter().map(|&(q, r)| axial_center(q, r)).collect();
        let half_extent = centers.iter().fold(Vec2::ZERO, |accumulator, center| {
            Vec2::new(
                accumulator.x.max(center.x.abs()),
                accumulator.y.max(center.y.abs()),
            )
        }) + Vec2::splat(HEX_RADIUS);

        let mut corner_positions: HashMap<CornerKey, Vec2> = HashMap::new();
        let mut cell_corners: Vec<[CornerKey; 6]> = Vec::with_capacity(axials.len());
        for center in &centers {
            let mut keys = [(0_i64, 0_i64); 6];
            for (corner, key) in keys.iter_mut().enumerate() {
                let base = *center + corner_offset(corner);
                *key = corner_key(base);
                corner_positions
                    .entry(*key)
                    .or_insert_with(|| perturbed_corner(base, *key, seed));
            }
            cell_corners.push(keys);
        }

        let index_of: HashMap<(i32, i32), usize> = axials
            .iter()
            .copied()
            .enumerate()
            .map(|(index, axial)| (axial, index))
            .collect();
        let neighbours: Vec<Vec<usize>> = axials
            .iter()
            .map(|&(q, r)| {
                DIRECTIONS
                    .iter()
                    .filter_map(|(delta, _)| index_of.get(&(q + delta.0, r + delta.1)).copied())
                    .collect()
            })
            .collect();

        let land = land_mask(&centers, half_extent, seed);
        let land_cells: Vec<usize> = (0..axials.len()).filter(|index| land[*index]).collect();
        let owners = assign_regions(
            &land_cells,
            &neighbours,
            &centers,
            region_countries,
            axials.len(),
        );

        let cells: Vec<MapCell> = axials
            .iter()
            .enumerate()
            .map(|(index, &axial)| {
                let polygon =
                    std::array::from_fn(|corner| corner_positions[&cell_corners[index][corner]]);
                let kind = owners[index].map_or(CellKind::Sea, |region| CellKind::Land { region });
                MapCell {
                    axial,
                    center: centers[index],
                    polygon,
                    kind,
                    tint: unit(hash_pair(
                        i64::from(axial.0),
                        i64::from(axial.1),
                        seed ^ 0x5EED,
                    )),
                    relief: relief_of(axial, centers[index], seed),
                }
            })
            .collect();

        let country_of = |region: usize| region_countries.get(region).copied().unwrap_or_default();
        let mut borders = Vec::new();
        for (index, &(q, r)) in axials.iter().enumerate() {
            for (delta, (first, second)) in DIRECTIONS {
                let neighbour = index_of.get(&(q + delta.0, r + delta.1)).copied();
                if let Some(other) = neighbour {
                    if other < index {
                        continue;
                    }
                }
                let here = owners[index];
                let there = neighbour.and_then(|other| owners[other]);
                let class = match (here, there) {
                    (None, None) => continue,
                    (Some(_), None) | (None, Some(_)) => BorderClass::Coast,
                    (Some(left), Some(right)) if left == right => BorderClass::Province,
                    (Some(left), Some(right)) if country_of(left) == country_of(right) => {
                        BorderClass::Region
                    }
                    (Some(_), Some(_)) => BorderClass::Country,
                };
                borders.push(BorderSegment {
                    from: corner_positions[&cell_corners[index][first]],
                    to: corner_positions[&cell_corners[index][second]],
                    class,
                    left: here,
                    right: there,
                });
            }
        }

        let region_count = region_countries.len();
        let mut region_cell_counts = vec![0_usize; region_count];
        let mut sums = vec![Vec2::ZERO; region_count];
        for cell in &cells {
            if let Some(region) = cell.region() {
                region_cell_counts[region] += 1;
                sums[region] += cell.center;
            }
        }
        let region_centroids: Vec<Vec2> = sums
            .iter()
            .zip(&region_cell_counts)
            .map(|(sum, count)| {
                if *count == 0 {
                    Vec2::ZERO
                } else {
                    *sum / *count as f32
                }
            })
            .collect();
        let region_capitals: Vec<Vec2> = region_centroids
            .iter()
            .enumerate()
            .map(|(region, centroid)| {
                cells
                    .iter()
                    .filter(|cell| cell.region() == Some(region))
                    .min_by(|left, right| {
                        left.center
                            .distance_squared(*centroid)
                            .total_cmp(&right.center.distance_squared(*centroid))
                    })
                    .map_or(*centroid, |cell| cell.center)
            })
            .collect();

        Self {
            cells,
            borders,
            region_centroids,
            region_capitals,
            region_cell_counts,
            half_extent,
        }
    }

    /// Returns the province containing a world-space point, if any.
    #[must_use]
    pub fn cell_at(&self, point: Vec2) -> Option<usize> {
        self.cells.iter().position(|cell| {
            cell.center.distance_squared(point) <= (HEX_RADIUS * 2.0).powi(2)
                && cell.contains(point)
        })
    }

    /// Returns the region owning the province at a world-space point.
    #[must_use]
    #[allow(dead_code)]
    pub fn region_at(&self, point: Vec2) -> Option<usize> {
        self.cell_at(point)
            .and_then(|index| self.cells[index].region())
    }
}

fn lattice_axials() -> Vec<(i32, i32)> {
    let mut axials = Vec::new();
    for q in -COLUMN_RANGE..=COLUMN_RANGE {
        let offset = -q.div_euclid(2);
        for r in (-ROW_RANGE + offset)..=(ROW_RANGE + offset) {
            axials.push((q, r));
        }
    }
    axials
}

fn axial_center(q: i32, r: i32) -> Vec2 {
    Vec2::new(
        HEX_RADIUS * 1.5 * q as f32,
        HEX_RADIUS * SQRT_3 * (r as f32 + q as f32 * 0.5),
    )
}

fn corner_offset(corner: usize) -> Vec2 {
    let angle = std::f32::consts::FRAC_PI_3 * corner as f32;
    Vec2::new(HEX_RADIUS * angle.cos(), HEX_RADIUS * angle.sin())
}

fn corner_key(point: Vec2) -> CornerKey {
    (
        (point.x * KEY_SCALE).round() as i64,
        (point.y * KEY_SCALE).round() as i64,
    )
}

fn perturbed_corner(base: Vec2, key: CornerKey, seed: u64) -> Vec2 {
    let horizontal = unit(hash_pair(key.0, key.1, seed ^ 0x00C0_FFEE));
    let vertical = unit(hash_pair(key.1, key.0, seed ^ 0x00BE_EF01));
    base + Vec2::new(horizontal - 0.5, vertical - 0.5) * 2.0 * HEX_RADIUS * CORNER_JITTER
}

fn land_mask(centers: &[Vec2], half_extent: Vec2, seed: u64) -> Vec<bool> {
    centers
        .iter()
        .map(|center| {
            let normalized = Vec2::new(center.x / half_extent.x, center.y / half_extent.y * 1.04);
            let radial = normalized.length();
            if radial > SEA_MARGIN {
                return false;
            }
            let coarse = value_noise(*center / (HEX_RADIUS * 7.0), seed ^ 0x0051);
            let fine = value_noise(*center / (HEX_RADIUS * 2.7), seed ^ 0x0A37);
            let shape = 0.64 * coarse + 0.36 * fine;
            radial < 0.46 + 0.66 * shape
        })
        .collect()
}

fn assign_regions(
    land_cells: &[usize],
    neighbours: &[Vec<usize>],
    centers: &[Vec2],
    region_countries: &[u32],
    cell_count: usize,
) -> Vec<Option<usize>> {
    let mut owners = vec![None; cell_count];
    if land_cells.is_empty() || region_countries.is_empty() {
        return owners;
    }
    let mut countries: Vec<u32> = Vec::new();
    for country in region_countries {
        if !countries.contains(country) {
            countries.push(*country);
        }
    }
    let country_seeds = farthest_point_seeds(land_cells, centers, countries.len());
    let country_owner = grow_partition(land_cells, neighbours, &country_seeds, centers, cell_count);

    for (slot, country) in countries.iter().enumerate() {
        let members: Vec<usize> = land_cells
            .iter()
            .copied()
            .filter(|cell| country_owner[*cell] == Some(slot))
            .collect();
        let regions: Vec<usize> = region_countries
            .iter()
            .enumerate()
            .filter(|(_, owner)| *owner == country)
            .map(|(region, _)| region)
            .collect();
        if members.is_empty() || regions.is_empty() {
            continue;
        }
        let seeds = farthest_point_seeds(&members, centers, regions.len());
        let local = grow_partition(&members, neighbours, &seeds, centers, cell_count);
        for cell in members {
            if let Some(index) = local[cell] {
                owners[cell] = regions.get(index).copied();
            }
        }
    }
    owners
}

fn farthest_point_seeds(candidates: &[usize], centers: &[Vec2], count: usize) -> Vec<usize> {
    let mut seeds: Vec<usize> = Vec::new();
    if candidates.is_empty() || count == 0 {
        return seeds;
    }
    let centroid = candidates
        .iter()
        .map(|cell| centers[*cell])
        .fold(Vec2::ZERO, |accumulator, center| accumulator + center)
        / candidates.len() as f32;
    let first = candidates
        .iter()
        .copied()
        .min_by(|left, right| {
            centers[*left]
                .distance_squared(centroid)
                .total_cmp(&centers[*right].distance_squared(centroid))
        })
        .unwrap_or(candidates[0]);
    seeds.push(first);
    while seeds.len() < count {
        let next = candidates.iter().copied().max_by(|left, right| {
            seed_distance(*left, &seeds, centers).total_cmp(&seed_distance(*right, &seeds, centers))
        });
        match next {
            Some(cell) if !seeds.contains(&cell) => seeds.push(cell),
            _ => break,
        }
    }
    seeds
}

fn seed_distance(cell: usize, seeds: &[usize], centers: &[Vec2]) -> f32 {
    seeds
        .iter()
        .map(|seed| centers[cell].distance_squared(centers[*seed]))
        .fold(f32::MAX, f32::min)
}

fn grow_partition(
    members: &[usize],
    neighbours: &[Vec<usize>],
    seeds: &[usize],
    centers: &[Vec2],
    cell_count: usize,
) -> Vec<Option<usize>> {
    let allowed: HashSet<usize> = members.iter().copied().collect();
    let mut owner: Vec<Option<usize>> = vec![None; cell_count];
    let mut queues: Vec<VecDeque<usize>> =
        seeds.iter().map(|seed| VecDeque::from([*seed])).collect();
    for (slot, seed) in seeds.iter().enumerate() {
        owner[*seed] = Some(slot);
    }
    let mut progressed = true;
    while progressed {
        progressed = false;
        #[allow(clippy::needless_range_loop)]
        for slot in 0..queues.len() {
            let Some(cell) = queues[slot].pop_front() else {
                continue;
            };
            progressed = true;
            for next in &neighbours[cell] {
                if allowed.contains(next) && owner[*next].is_none() {
                    owner[*next] = Some(slot);
                    queues[slot].push_back(*next);
                }
            }
        }
    }
    for cell in members {
        if owner[*cell].is_none() {
            owner[*cell] = seeds
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    centers[*cell]
                        .distance_squared(centers[**left])
                        .total_cmp(&centers[*cell].distance_squared(centers[**right]))
                })
                .map(|(slot, _)| slot);
        }
    }
    owner
}

fn relief_of(axial: (i32, i32), center: Vec2, seed: u64) -> Relief {
    let ridge = value_noise(center / (HEX_RADIUS * 4.1), seed ^ 0x00A1);
    let local = unit(hash_pair(
        i64::from(axial.0),
        i64::from(axial.1),
        seed ^ 0x00D5,
    ));
    let score = 0.7 * ridge + 0.3 * local;
    if score > 0.78 {
        Relief::Mountains
    } else if score > 0.66 {
        Relief::Hills
    } else if score < 0.34 {
        Relief::Forest
    } else {
        Relief::Plain
    }
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let mut previous = polygon.len() - 1;
    for current in 0..polygon.len() {
        let (a, b) = (polygon[current], polygon[previous]);
        if (a.y > point.y) != (b.y > point.y) {
            let ratio = (point.y - a.y) / (b.y - a.y);
            if point.x < a.x + ratio * (b.x - a.x) {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}

const fn splitmix64(value: u64) -> u64 {
    let seeded = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let first = (seeded ^ (seeded >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let second = (first ^ (first >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    second ^ (second >> 31)
}

fn hash_pair(first: i64, second: i64, salt: u64) -> u64 {
    let first = u64::from_ne_bytes(first.to_ne_bytes());
    let second = u64::from_ne_bytes(second.to_ne_bytes());
    let mixed = splitmix64(first ^ 0x1234_5678_9ABC_DEF0);
    splitmix64(mixed ^ splitmix64(second).rotate_left(17) ^ salt)
}

fn unit(hash: u64) -> f32 {
    (hash >> 40) as f32 / 16_777_216.0
}

fn value_noise(point: Vec2, salt: u64) -> f32 {
    let base_x = point.x.floor();
    let base_y = point.y.floor();
    let fraction_x = smoothstep(point.x - base_x);
    let fraction_y = smoothstep(point.y - base_y);
    let x = base_x as i64;
    let y = base_y as i64;
    let bottom_left = unit(hash_pair(x, y, salt));
    let bottom_right = unit(hash_pair(x + 1, y, salt));
    let top_left = unit(hash_pair(x, y + 1, salt));
    let top_right = unit(hash_pair(x + 1, y + 1, salt));
    let bottom = bottom_left + (bottom_right - bottom_left) * fraction_x;
    let top = top_left + (top_right - top_left) * fraction_x;
    bottom + (top - bottom) * fraction_y
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNTRIES: [u32; 3] = [1, 1, 2];
    const SEED: u64 = 0x0ADA_1234_5678_9ABC;

    fn sample_map() -> StrategicMap {
        StrategicMap::generate(&COUNTRIES, SEED)
    }

    #[test]
    fn generation_is_deterministic() {
        assert_eq!(sample_map(), sample_map());
    }

    #[test]
    fn continent_covers_a_plausible_share_of_the_lattice() {
        let map = sample_map();
        let land = map
            .cells
            .iter()
            .filter(|cell| cell.region().is_some())
            .count();
        let ratio = land as f32 / map.cells.len() as f32;
        assert!(ratio > 0.25, "continent too small: {ratio}");
        assert!(ratio < 0.80, "continent too large: {ratio}");
    }

    #[test]
    fn every_region_owns_a_contiguous_area() {
        let map = sample_map();
        let index_of: HashMap<(i32, i32), usize> = map
            .cells
            .iter()
            .enumerate()
            .map(|(index, cell)| (cell.axial, index))
            .collect();
        for region in 0..COUNTRIES.len() {
            let members: HashSet<usize> = map
                .cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.region() == Some(region))
                .map(|(index, _)| index)
                .collect();
            assert!(!members.is_empty(), "region {region} has no provinces");
            let start = *members.iter().min().expect("member");
            let mut seen = HashSet::from([start]);
            let mut queue = VecDeque::from([start]);
            while let Some(cell) = queue.pop_front() {
                let (q, r) = map.cells[cell].axial;
                for (delta, _) in DIRECTIONS {
                    let Some(next) = index_of.get(&(q + delta.0, r + delta.1)).copied() else {
                        continue;
                    };
                    if members.contains(&next) && seen.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
            assert_eq!(seen.len(), members.len(), "region {region} is fragmented");
        }
    }

    #[test]
    fn province_centers_hit_test_to_themselves() {
        let map = sample_map();
        for (index, cell) in map.cells.iter().enumerate() {
            assert_eq!(map.cell_at(cell.center), Some(index));
        }
    }

    #[test]
    fn neighbouring_provinces_share_exact_border_endpoints() {
        let map = sample_map();
        let mut corners: HashMap<CornerKey, Vec2> = HashMap::new();
        for cell in &map.cells {
            for corner in cell.polygon {
                let key = corner_key(corner - Vec2::splat(0.0));
                if let Some(existing) = corners.insert(key, corner) {
                    assert!(existing.distance(corner) < 1.0e-3);
                }
            }
        }
        assert!(map.borders.iter().all(|border| border.from != border.to));
    }

    #[test]
    fn country_borders_only_separate_different_countries() {
        let map = sample_map();
        for border in &map.borders {
            if border.class == BorderClass::Country {
                let left = COUNTRIES[border.left.expect("left region")];
                let right = COUNTRIES[border.right.expect("right region")];
                assert_ne!(left, right);
            }
        }
    }
}
