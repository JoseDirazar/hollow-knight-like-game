use bevy::prelude::*;

/// Calculates the distance between two points in 2D space
pub fn distance_between_points(a: Vec2, b: Vec2) -> f32 {
    a.distance(b)
}

/// Checks if two rectangles (hitboxes) are colliding using AABB collision detection
pub fn check_rect_collision(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    let half_size1 = size1 / 2.0;
    let half_size2 = size2 / 2.0;

    (pos1.x - half_size1.x < pos2.x + half_size2.x)
        && (pos1.x + half_size1.x > pos2.x - half_size2.x)
        && (pos1.y - half_size1.y < pos2.y + half_size2.y)
        && (pos1.y + half_size1.y > pos2.y - half_size2.y)
}

/// Checks if a point is within a rectangle
pub fn point_in_rect(point: Vec2, rect_pos: Vec2, rect_size: Vec2) -> bool {
    let half_size = rect_size / 2.0;
    point.x >= rect_pos.x - half_size.x
        && point.x <= rect_pos.x + half_size.x
        && point.y >= rect_pos.y - half_size.y
        && point.y <= rect_pos.y + half_size.y
}

/// Calculates the direction vector from point a to point b
pub fn direction_vector(from: Vec2, to: Vec2) -> Vec2 {
    (to - from).normalize()
}

/// Checks if an entity is within a certain range of another entity
pub fn is_within_range(pos1: Vec2, pos2: Vec2, range: f32) -> bool {
    distance_between_points(pos1, pos2) <= range
}

/// Calculates the angle between two vectors in radians
pub fn angle_between_vectors(a: Vec2, b: Vec2) -> f32 {
    a.angle_to(b)
}

/// Linearly interpolates between two values
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamps a value between a minimum and maximum
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Converts degrees to radians
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

/// Converts radians to degrees
pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}
