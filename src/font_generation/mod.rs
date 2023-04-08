use glam::{uvec2, UVec2};
use rust_webgl2::{
    GlTexture2D, Graphics, Texture2DProps, TextureInternalFormat,
};

pub struct CharacterTextureSlice {
    index: usize,
    extent: (UVec2, UVec2),
}

pub struct CharacterBitmap {
    bitmap: Vec<u8>,
    size: UVec2,
}

pub struct CharacterInfo {
    pub character: char,
    pub metrics: fontdue::Metrics,
}

pub fn parse_font_from_bytes(
    font_bytes: &[u8],
    scale: f32,
    collection_index: u32,
) -> fontdue::FontResult<fontdue::Font> {
    fontdue::Font::from_bytes(
        font_bytes,
        fontdue::FontSettings {
            scale,
            collection_index,
        },
    )
}

pub struct FontCharacters {
    pub character_bitmaps: Vec<CharacterBitmap>,
    pub character_info_collection: Vec<CharacterInfo>,
    pub size_factor: f32,
}

impl FontCharacters {
    pub fn new_from_file(
        file_data: &[u8],
        character_size: f32,
        font_char_limit: Option<usize>,
    ) -> Result<Self, ()> {
        match parse_font_from_bytes(file_data, character_size, 0) {
            Ok(font) => {
                let character_collection =
                    create_character_slices(&font, character_size, font_char_limit);

                let character_bitmaps =
                    create_character_bitmaps(&font, &character_collection, character_size);

                Ok(Self {
                    character_bitmaps,
                    character_info_collection: character_collection,
                    size_factor: 1.0 / character_size,
                })
            }
            Err(_err) => return Err(()),
        }
    }
}

fn create_character_slices(
    font: &fontdue::Font,
    character_size: f32,
    font_char_limit: Option<usize>,
) -> Vec<CharacterInfo> {
    let mut char_array: Vec<char> = font.chars().keys().into_iter().map(|key| *key).collect();
    char_array.sort();
    match font_char_limit {
        None => { /* No Op */ }
        Some(count) => {
            char_array.truncate(count);
        }
    };

    let character_count = char_array.len();
    let mut slice_coords = Vec::<CharacterInfo>::with_capacity(character_count);

    for char_ref in char_array.iter() {
        let character = *char_ref;
        let char_index = font.lookup_glyph_index(character);
        let metrics = font.metrics_indexed(char_index, character_size);
        slice_coords.push(CharacterInfo {
            character,
            metrics,
        });
    }
    slice_coords.sort_by(|a, b| a.metrics.height.cmp(&b.metrics.height));

    slice_coords
}

fn create_character_bitmaps(
    font: &fontdue::Font,
    characters: &Vec<CharacterInfo>,
    character_size: f32,
) -> Vec<CharacterBitmap> {
    let mut bitmaps: Vec<CharacterBitmap> = Vec::<CharacterBitmap>::with_capacity(characters.len());
    for character in characters.iter() {
        //Get character bitmap
        let char_index = font.lookup_glyph_index(character.character);

        //Add glyph data to texture
        font.metrics_indexed(char_index, character_size);
        let (_, bitmap) = font.rasterize_indexed(char_index, character_size);

        let chacter_bitmap = CharacterBitmap {
            bitmap,
            size: uvec2(
                character.metrics.width as u32,
                character.metrics.height as u32,
            ),
        };
        bitmaps.push(chacter_bitmap);
    }
    bitmaps
}

pub fn generate_font_texture(
    font: &FontCharacters,
    texture_size: UVec2,
    padding: u32,
) -> (Vec<u8>, Vec<CharacterTextureSlice>) {
    let mut character_size: Vec<(usize, UVec2)> = font
        .character_bitmaps
        .iter()
        .enumerate()
        .map(|(index, char)| (index, char.size))
        .collect();
    character_size.sort_by(|(_, size_a), (_, size_b)| size_a.y.cmp(&size_b.y));
    let mut font_texture = vec![0 as u8; (texture_size.x * texture_size.y) as usize];
    let mut character_texture_slices = Vec::new();

    let mut coord = uvec2(0, 0);
    let mut current_max_height = 0;
    for (char_index, _) in character_size.iter() {
        let character = &font.character_bitmaps[*char_index];

        //Validate buffer
        let buffer_size = character.bitmap.len();
        let reported_buffer_size = (character.size.x * character.size.y) as usize;
        assert!(buffer_size == reported_buffer_size);

        //Check if bitmap can be placed in this coordinate
        if !check_bitmap_fits(texture_size, coord, character.size, padding) {
            coord.x = 0;
            coord.y += current_max_height;
            current_max_height = 0;
            if !check_bitmap_fits(texture_size, coord, character.size, padding) {
                panic!(
                    "Can't generate texture | coord: {:?} | char_size: {:?}",
                    coord, character.size
                );
            }
        }

        //Generate character texture slice
        let char_slice = CharacterTextureSlice {
            index: *char_index,
            extent: (
                coord,
                coord + character.size + uvec2(padding * 2, padding * 2),
            ),
        };
        character_texture_slices.push(char_slice);

        //Update the current line hight
        let char_height = character.size.y + padding * 2;
        if current_max_height < char_height {
            current_max_height = char_height;
        }

        copy_bitmap_to_texture(&mut font_texture, texture_size, character, coord, padding);
        coord.x += character.size.x + padding * 2;
    }

    (font_texture, character_texture_slices)
}

/// Checks if a bitmap fits in a texture
pub fn check_bitmap_fits(
    texture_size: UVec2,
    coord: UVec2,
    bitmap_size: UVec2,
    padding: u32,
) -> bool {
    let extent_x = coord.x + bitmap_size.x + padding * 2;
    let extent_y = coord.y + bitmap_size.y + padding * 2;
    (extent_x < texture_size.x) && (extent_y < texture_size.y)
}

/// Converts a coordinate to an index in a 1D array
pub fn coord_to_index(size: UVec2, coord: UVec2) -> usize {
    (coord.x + coord.y * size.x) as usize
}

/// Copies a bitmap to a texture
pub fn copy_bitmap_to_texture(
    texture: &mut Vec<u8>,
    texture_size: UVec2,
    char_bitmap: &CharacterBitmap,
    coord: UVec2,
    padding: u32,
) {
    for i in 0..char_bitmap.size.x {
        for j in 0..char_bitmap.size.y {
            let c_coord = uvec2(i, j);
            let c_index = coord_to_index(char_bitmap.size, c_coord);
            let c_pixel = char_bitmap.bitmap[c_index];
            //let c_pixel = if c_pixel > 126 { 255 } else { 0 };
            let p_coord = uvec2(padding, padding) + c_coord + coord;
            let p_index = coord_to_index(texture_size, p_coord);
            texture[p_index] = c_pixel;
        }
    }
}

/// Generates a texture and a vector of character texture slices
pub fn generate_gl_texture_2d(
    graphics: &Graphics,
    font: &FontCharacters,
    texture_size: UVec2,
    padding: u32,
) -> (GlTexture2D, Vec<CharacterTextureSlice>) {
    let (texture_data, character_slices) = generate_font_texture(font, texture_size, padding);
    let gl_texture = GlTexture2D::new(
        graphics,
        Texture2DProps::clamped_linear_no_mipmap(),
        texture_size,
        TextureInternalFormat::R8,
        None,
        Some("FontTexture".into())
    )
    .unwrap();
    gl_texture.set_texture_data(0, &texture_data, 0).unwrap();
    (gl_texture, character_slices)
}