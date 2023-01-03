use glam::*;

use crate::slotmap::prelude::*;
crate::slotmap::prelude::create_custom_key!(BoneKey);

pub struct Skeleton{
	transform: Mat4,
	bones: Slotmap<Bone>,
	bone_tree: BoneNode,
}
pub struct BoneNode{
	node: BoneKey,
	children: Vec<BoneKey>
}
pub struct Bone{
	transform: Mat4
}
