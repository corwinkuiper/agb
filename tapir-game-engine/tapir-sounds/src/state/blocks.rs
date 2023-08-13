use std::{borrow::Cow, collections::HashMap};

mod fundamental_shape;

use crate::state;

use self::fundamental_shape::{FundamentalShapeBlock, FundamentalShapeType};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BlockCategory {
    Fundamental,
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[non_exhaustive]
pub struct BlockName {
    pub category: BlockCategory,
    pub name: String,
}

type MakeBlockType = Box<dyn Fn() -> Box<dyn BlockType>>;

pub struct BlockFactory {
    creation_functions: HashMap<BlockName, MakeBlockType>,
}

impl BlockFactory {
    pub fn new() -> Self {
        let mut creation_functions: HashMap<BlockName, MakeBlockType> = HashMap::new();

        for fundamental_shape in FundamentalShapeType::all() {
            creation_functions.insert(
                BlockName {
                    category: BlockCategory::Fundamental,
                    name: fundamental_shape.name().to_owned(),
                },
                Box::new(move || Box::new(FundamentalShapeBlock::new(fundamental_shape))),
            );
        }

        Self { creation_functions }
    }

    pub fn available_blocks(&self) -> impl Iterator<Item = &BlockName> + '_ {
        let mut names = self.creation_functions.keys().collect::<Vec<_>>();
        names.sort();

        names.into_iter()
    }

    pub fn make_block(&self, name: &BlockName, pos: (f32, f32)) -> Block {
        let block_type = self
            .creation_functions
            .get(name)
            .unwrap_or_else(|| panic!("Failed to make block with name {name:?}"));

        Block::new(block_type(), pos)
    }
}

#[derive(Clone)]
pub struct Block {
    block_type: Box<dyn BlockType>,
    id: state::Id,
    x: f32,
    y: f32,
    dirty: bool,
}

#[derive(Clone, Debug)]
pub enum Input {
    Toggle(bool),
    Frequency(f64),
    Amplitude(f64),
    Periods(f64),
}

impl Block {
    pub fn new(block_type: Box<dyn BlockType>, pos: (f32, f32)) -> Self {
        Self {
            block_type,
            x: pos.0,
            y: pos.1,
            id: state::Id::new(),
            dirty: true,
        }
    }

    pub fn name(&self) -> Cow<'static, str> {
        self.block_type.name()
    }

    pub fn id(&self) -> state::Id {
        self.id
    }

    pub fn inputs(&self) -> Vec<(Cow<'static, str>, Input)> {
        self.block_type.inputs()
    }

    pub fn set_input(&mut self, index: usize, value: &Input) {
        self.block_type.set_input(index, value);
        self.dirty = true;
    }

    pub fn calculate(&self, global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64> {
        self.block_type.calculate(global_frequency, inputs)
    }

    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn pos_delta(&mut self, delta: (f32, f32)) {
        self.x += delta.0;
        self.y += delta.1;
        // doesn't set dirty because it doesn't change the output
    }

    pub(super) fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(super) fn clean(&mut self) {
        self.dirty = false;
    }
}

pub trait BlockClone {
    fn clone_box(&self) -> Box<dyn BlockType>;
}

pub trait BlockType: BlockClone + Send + Sync {
    fn name(&self) -> Cow<'static, str>;
    fn inputs(&self) -> Vec<(Cow<'static, str>, Input)>;
    fn set_input(&mut self, index: usize, value: &Input);
    fn calculate(&self, global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64>;
}

impl Clone for Box<dyn BlockType> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<T> BlockClone for T
where
    T: 'static + BlockType + Clone,
{
    fn clone_box(&self) -> Box<dyn BlockType> {
        Box::new(self.clone())
    }
}

fn stretch_frequency_shift(input: f64) -> f64 {
    1.0 / (1.0 - input)
}
