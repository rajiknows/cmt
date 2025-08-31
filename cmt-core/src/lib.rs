use crate::utils::calculate_merkle_hash;

mod utils;

pub type Key = Vec<u8>;
pub type Priority = u128;
pub type Hash = Vec<u8>;

impl PartialOrd for Key {
    fn gt(&self, other: &Self) -> bool {}
}

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

    pub fn insert(&mut self, key: K, value: V) {
        // first insert = root
        if self.root.is_none() {
            let hash = calculate_merkle_hash(&key, &Vec::new(), &Vec::new());
            let priority = find_priority(&key);
            self.root = Some(Box::new(TreeNode {
                key,
                priority,
                value,
                hash,
                left: None,
                right: None,
            }));
            return;
        }

        // walk down the tree
        let mut node = self.root.as_mut().unwrap();
        loop {
            if key < node.key {
                if let Some(left) = node.left.as_mut() {
                    node = left;
                } else {
                    let hash = calculate_merkle_hash(&key, &Vec::new(), &Vec::new());
                    let priority = find_priority(&key);
                    node.left = Some(Box::new(TreeNode {
                        key,
                        priority,
                        value,
                        hash,
                        left: None,
                        right: None,
                    }));
                    // TODO: check priority vs node.priority, rotate if needed
                    break;
                }
            } else if key > node.key {
                if let Some(right) = node.right.as_mut() {
                    node = right;
                } else {
                    let hash = calculate_merkle_hash(&key, &Vec::new(), &Vec::new());
                    let priority = find_priority(&key);
                    node.right = Some(Box::new(TreeNode {
                        key,
                        priority,
                        value,
                        hash,
                        left: None,
                        right: None,
                    }));
                    // TODO: check priority vs node.priority, rotate if needed
                    break;
                }
            } else {
                // key == node.key → update value
                node.value = value;
                break;
            }
        }
    }
}

pub struct Proof {
    pub key: Key,
    pub value_hash: Hash,
    pub path: Vec<Hash>,
    pub existence: bool,
}

fn find_priority<K: AsRef<[u8]>>(key: &K) -> Priority {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(key.as_ref());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    u128::from_be_bytes(bytes)
}
