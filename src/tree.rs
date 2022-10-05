#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Position {
    Left,
    Right,
}

#[derive(Debug)]
pub struct BinaryTree<T> {
    pub value: T,
    left: Option<Box<BinaryTree<T>>>,
    right: Option<Box<BinaryTree<T>>>,
}

impl<T> BinaryTree<T> {
    pub fn new(value: T, left: Option<BinaryTree<T>>, right: Option<BinaryTree<T>>) -> Self {
        Self {
            value,
            left: left.map(|node| Box::new(node)),
            right: right.map(|node| Box::new(node)),
        }
    }

    pub fn left(&self) -> Option<&BinaryTree<T>> {
        Some(self.left.as_ref()?)
    }

    pub fn right(&self) -> Option<&BinaryTree<T>> {
        Some(self.right.as_ref()?)
    }

    pub fn left_mut(&mut self) -> Option<&mut BinaryTree<T>> {
        Some(self.left.as_mut()?)
    }

    pub fn right_mut(&mut self) -> Option<&mut BinaryTree<T>> {
        Some(self.right.as_mut()?)
    }
}
