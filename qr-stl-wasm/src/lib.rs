mod utils;

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn qr2stl(
    input: String,
    base_height: f32,
    base_size: f32,
    pixel_size: f32,
) -> Result<Vec<u8>, JsValue> {
    let mesh_opts = qr_stl::MeshOptions {
        base_height,
        base_size,
        pixel_size,
    };
    let tris = qr_stl::qr_to_triangles(input.as_bytes(), &mesh_opts)
        .map_err(|_| "Failed to generate triangles")?;

    let mut res = Vec::new();
    qr_stl::save_stl(&tris, &mut res).map_err(|_| "Failed to save STL to buffer")?;
    Ok(res)
}
