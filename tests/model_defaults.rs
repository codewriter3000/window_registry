use window_registry::WindowGeometry;

#[test]
fn window_geometry_default_is_zeroed() {
    let geom = WindowGeometry::default();
    assert_eq!(geom.x, 0);
    assert_eq!(geom.y, 0);
    assert_eq!(geom.width, 0);
    assert_eq!(geom.height, 0);
}
