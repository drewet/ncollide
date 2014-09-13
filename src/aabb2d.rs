extern crate nalgebra;
extern crate "ncollide2df32" as ncollide;

use nalgebra::na::{Vec2, Iso2};
use nalgebra::na;
use ncollide::bounding_volume::{BoundingVolume, LooseBoundingVolume, HasAABB};
use ncollide::geom::{Cone, Cylinder};

fn main() {
    /*
     * Initialize the shapes.
     */
    let cone     = Cone::new(0.5, 0.5);
    let cylinder = Cylinder::new(1.0, 0.5);

    let cone_pos     = Iso2::new(Vec2::y(), na::zero()); // 1.0 along the `y` axis.
    let cylinder_pos = na::one();                        // Identity matrix.

    /*
     * Compute their bounding spheres.
     */
    let aabb_cone     = cone.aabb(&cone_pos);
    let aabb_cylinder = cylinder.aabb(&cylinder_pos);

    // Merge the two spheres.
    let bounding_aabb = aabb_cone.merged(&aabb_cylinder);

    // Enlarge the cylinder bounding sphere.
    let loose_aabb_cylinder = aabb_cylinder.loosened(1.0);

    // Intersection and inclusion tests.
    assert!(aabb_cone.intersects(&aabb_cylinder));
    assert!(bounding_aabb.contains(&aabb_cone));
    assert!(bounding_aabb.contains(&aabb_cylinder));
    assert!(!aabb_cylinder.contains(&bounding_aabb));
    assert!(!aabb_cone.contains(&bounding_aabb));
    assert!(loose_aabb_cylinder.contains(&aabb_cylinder));
}
