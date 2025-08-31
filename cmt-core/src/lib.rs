mod utils;

pub type Key = Vec<u8>;
pub type Priority = u128;
pub type Hash = Vec<u8>;

pub trait Hasher {
    fn hash(data: &[u8]) -> Hash;
}

pub struct TreeNode<K, V> {
    pub key: K,
    pub priority: Priority,
    pub value: V,
    pub hash: Hash,
    pub left: Option<Box<TreeNode<K, V>>>,
    pub right: Option<Box<TreeNode<K, V>>>,
}

pub struct CartesianMerkleTree<K, V> {
    root: Option<Box<TreeNode<K, V>>>,
}

impl<K, V> CartesianMerkleTree<K, V> {
    pub fn new(root: Option<Box<TreeNode<K, V>>>) -> Self {
        Self { root }
    }
}

pub struct Proof {
    pub key: Key,
    pub value_hash: Hash,
    pub path: Vec<Hash>,
    pub existence: bool,
}
