
use crate::{js_file_fetcher::*, mesh::{gltf2::{generate_all_mesh_from_gltf, Gltf2Mesh}}};

pub fn request_file_cors_no_cache_same_origin(file_fetcher: &mut FileFetcher, content_type: &str, file_path: &str) -> Uuid {
    let file_id = Uuid::new_v4();
    let mut request = web_sys::RequestInit::new();
    request.method("GET");
    request.mode(web_sys::RequestMode::Cors);
    request.cache(web_sys::RequestCache::NoCache);
    request.credentials(web_sys::RequestCredentials::SameOrigin);
    
    let mut headers = HashMap::new();
    headers.insert("Content-Type", content_type);

    let header_val = serde_wasm_bindgen::to_value(&headers)
        .expect("Error while trying to generate the header {:?}");
    request.headers(&header_val);
    file_fetcher.request_file(file_id, request, file_path);
    file_id
}

pub fn request_gltf_file(file_fetcher: &mut FileFetcher, file_path: &str) -> Uuid {
    let file_id = Uuid::new_v4();
    let mut request = web_sys::RequestInit::new();
    request.method("GET");
    request.mode(web_sys::RequestMode::Cors);
    request.cache(web_sys::RequestCache::NoCache);
    request.credentials(web_sys::RequestCredentials::SameOrigin);
    let mut headers = HashMap::new();
    headers.insert("Content-Type", "model/gltf-binary");

    let header_val = serde_wasm_bindgen::to_value(&headers)
        .expect("Error while trying to generate the header {:?}");
    request.headers(&header_val);
    file_fetcher.request_file(file_id, request, file_path);
    file_id
}

pub enum GltfFileLoader {
    Request(FileFetchRequest),
    GltfMesh(Vec<Gltf2Mesh>),
}
impl GltfFileLoader {
    pub fn new(file_url: String, file_fetcher: &mut FileFetcher) -> Self {
        let file_id = request_gltf_file(file_fetcher, &file_url);
        Self::Request(FileFetchRequest::InQueue(file_id))
    }
    pub fn is_request(&self) -> bool {
        match self {
            GltfFileLoader::Request(_) => true,
            GltfFileLoader::GltfMesh(_) => false,
        }
    }
    pub fn get_mesh_vec(&self) -> Option<&Vec<Gltf2Mesh>>{
        match self{
            GltfFileLoader::Request(_) => None,
            GltfFileLoader::GltfMesh(mesh_vec) => Some(mesh_vec),
        }
    }
    pub fn try_transform(
        &mut self,
        file_fetcher: &FileFetcher,
    ) -> Result<(), FetchError> {
        match self {
            GltfFileLoader::Request(request) => match request.try_transform(file_fetcher) {
                Ok(_) => {
					match request {
						FileFetchRequest::InQueue(_) => {
							Ok(())
						},
						FileFetchRequest::Data(data) => {
							let mesh_vec = generate_all_mesh_from_gltf(data)
								.expect("Cannot generate meshes");
							*self = Self::GltfMesh(mesh_vec);
							Ok(())
						}
					}
				},
                Err(error) => Err(error),
            },
            GltfFileLoader::GltfMesh(_) => Ok(()),
        }
    }
}
