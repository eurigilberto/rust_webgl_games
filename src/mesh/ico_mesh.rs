pub struct IcoMeshObject {
    pub mesh: SimpleIcoMesh,
    pub vertex_array_object: GlVertexArrayObject,
    pub material: GlMaterial,
    pub model_transform_index: UniformIndex,
}

impl IcoMeshObject {
    pub fn simple(graphics: &Graphics, edge_length: f32) -> Self {
        let mesh = SimpleIcoMesh::new(&graphics, edge_length).expect("Create Mesh Error");
        IcoMeshObject::new(graphics, mesh)
    }

    pub fn subdivided(graphics: &Graphics, edge_length: f32, subdiv: u32) -> Self {
        let mesh = SimpleIcoMesh::with_subdivision(graphics, edge_length, subdiv)
            .expect("Create Mesh Error");
        IcoMeshObject::new(graphics, mesh)
    }

    fn new(graphics: &Graphics, mesh: SimpleIcoMesh) -> Self {
        let vertex_array_object = SimpleIcoMesh::create_vertex_array_object(
            &graphics,
            &mesh.position_buffer,
            &mesh.normal_buffer,
        )
        .expect("Create VAO Error");
        let material = simple_ico_material(&graphics).expect("Create Material Error");
        let model_transform_index = material
            .program
            .uniforms
            .get_uniform_index("model_transform")
            .expect("Uniform Index");

        Self {
            mesh,
            vertex_array_object,
            material,
            model_transform_index,
        }
    }

    pub fn set_model_transform(&mut self, transform: Mat4) {
        self.material.program.uniforms.set_uniform(
            self.model_transform_index,
            FloatUniform::Mat4(transform).into(),
        );
    }

    pub fn render(&mut self, graphics: &Graphics) {
        self.material.set_capabilities(graphics, 0);
        let mut current_program = self.material.program.use_program();
        self.vertex_array_object.bind();
        current_program.push_all_uniforms();
        current_program.draw_arrays(PrimitiveType::TRIANGLES, 0, self.mesh.vertex_count);
        self.vertex_array_object.unbind();
    }
}