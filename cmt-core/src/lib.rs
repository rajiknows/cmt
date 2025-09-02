use std::clone;

use crate::utils::calculate_merkle_hash;

mod utils;

pub type Key = [u8; 32];
pub type Priority = i128;
pub type Hash = Vec<u8>;

pub trait Hasher {
    fn hash(data: &[u8]) -> Hash;
}

#[derive(Debug, Clone)]
pub struct TreeNode<K, V> {
    pub key: K,
    pub priority: Priority,
    pub value: V,
    pub hash: Hash,
    pub left: Option<Box<TreeNode<K, V>>>,
    pub right: Option<Box<TreeNode<K, V>>>,
}

impl<K: PartialEq, V> PartialEq for TreeNode<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<K: Eq, V> Eq for TreeNode<K, V> {}

pub struct CartesianMerkleTree<K, V> {
    root: Option<Box<TreeNode<K, V>>>,
}

impl<K: std::cmp::PartialEq + std::cmp::PartialOrd + Clone, V> CartesianMerkleTree<K, V> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        let mut cur = self.root.as_ref();
        while let Some(n) = cur {
            if &n.key == key {
                return true;
            } else if key < &n.key {
                cur = n.left.as_ref();
            } else {
                cur = n.right.as_ref();
            }
        }
        false
    }

    pub fn insert(&mut self, key: K, value: V)
    where
        K: PartialOrd + Ord + AsRef<[u8]> + Clone,
        V: Clone,
    {
        let priority = find_priority(&key);
        self.root = Self::insert_recursive(self.root.take(), key, value, priority);
    }

    fn insert_recursive(
        node: Option<Box<TreeNode<K, V>>>,
        key: K,
        value: V,
        priority: Priority,
    ) -> Option<Box<TreeNode<K, V>>>
    where
        K: PartialOrd + Ord + AsRef<[u8]> + Clone,
        V: Clone,
    {
        let mut current_node = match node {
            Some(n) => n,
            None => {
                let hash = calculate_merkle_hash(&key, &Vec::new(), &Vec::new());
                return Some(Box::new(TreeNode {
                    key,
                    priority,
                    value,
                    hash,
                    left: None,
                    right: None,
                }));
            }
        };

        if priority > current_node.priority {
            let hash = calculate_merkle_hash(&key, &Vec::new(), &Vec::new());
            let mut new_node = Box::new(TreeNode {
                key,
                priority,
                value,
                hash,
                left: None,
                right: None,
            });
            Self::split(
                &mut current_node,
                &new_node.key,
                &mut new_node.left,
                &mut new_node.right,
            );
            // recompute hash for new_node
            let left_hash = new_node
                .left
                .as_ref()
                .map(|n| n.hash.clone())
                .unwrap_or_default();
            let right_hash = new_node
                .right
                .as_ref()
                .map(|n| n.hash.clone())
                .unwrap_or_default();
            new_node.hash = calculate_merkle_hash(&new_node.key, &left_hash, &right_hash);
            return Some(new_node);
        }

        if key < current_node.key {
            current_node.left =
                Self::insert_recursive(current_node.left.take(), key, value, priority);
        } else if key > current_node.key {
            current_node.right =
                Self::insert_recursive(current_node.right.take(), key, value, priority);
        } else {
            current_node.value = value;
        }

        let left_hash = current_node
            .left
            .as_ref()
            .map(|n| n.hash.clone())
            .unwrap_or_default();
        let right_hash = current_node
            .right
            .as_ref()
            .map(|n| n.hash.clone())
            .unwrap_or_default();
        current_node.hash = calculate_merkle_hash(&current_node.key, &left_hash, &right_hash);

        Some(current_node)
    }

    fn split(
        node: &mut Box<TreeNode<K, V>>,
        key: &K,
        left: &mut Option<Box<TreeNode<K, V>>>,
        right: &mut Option<Box<TreeNode<K, V>>>,
    ) where
        K: PartialOrd + Ord + AsRef<[u8]> + Clone,
        V: Clone,
    {
        if node.key < *key {
            *left = Some(node.clone());
            if let Some(right_child) = node.right.as_mut() {
                Self::split(right_child, key, left, right);
            }
        } else {
            *right = Some(node.clone());
            if let Some(left_child) = node.left.as_mut() {
                Self::split(left_child, key, left, right);
            }
        }
    }
    pub fn remove(&mut self, key: &K)
    where
        K: PartialOrd + Ord + AsRef<[u8]>,
    {
        self.root = Self::remove_recursive(self.root.take(), key);
    }

    fn remove_recursive(node: Option<Box<TreeNode<K, V>>>, key: &K) -> Option<Box<TreeNode<K, V>>>
    where
        K: PartialOrd + Ord + AsRef<[u8]>,
    {
        if let Some(mut current_node) = node {
            if *key < current_node.key {
                current_node.left = Self::remove_recursive(current_node.left.take(), key);
            } else if *key > current_node.key {
                current_node.right = Self::remove_recursive(current_node.right.take(), key);
            } else {
                // Node found, set priority to -inf and heapify down
                current_node.priority = i128::MIN;
                return Self::heapify(current_node);
            }
            // Update hash
            let left_hash = current_node
                .left
                .as_ref()
                .map(|n| n.hash.clone())
                .unwrap_or_default();
            let right_hash = current_node
                .right
                .as_ref()
                .map(|n| n.hash.clone())
                .unwrap_or_default();
            current_node.hash = calculate_merkle_hash(&current_node.key, &left_hash, &right_hash);
            return Some(current_node);
        }
        None
    }

    fn heapify(node: Box<TreeNode<K, V>>) -> Option<Box<TreeNode<K, V>>>
    where
        K: PartialOrd + Ord + AsRef<[u8]>,
    {
        if node.left.is_none() && node.right.is_none() {
            // Leaf node, remove it
            return None;
        }

        let left_priority = node.left.as_ref().map_or(i128::MIN, |n| n.priority);
        let right_priority = node.right.as_ref().map_or(i128::MIN, |n| n.priority);

        if left_priority > right_priority {
            let mut new_node = utils::rotate_right(node);
            new_node.right = Self::heapify(new_node.right.take().unwrap());
            Some(new_node)
        } else {
            let mut new_node = utils::rotate_left(node);
            new_node.left = Self::heapify(new_node.left.take().unwrap());
            Some(new_node)
        }
    }

    pub fn generate_proof(&self, key: &K) -> Proof<K>
    where
        K: Clone,
    {
        let mut prefix: Vec<(K, Hash)> = Vec::new();
        let mut cur = self.root.as_ref();
        let mut last: Option<&TreeNode<K, V>> = None;
        let mut existence = false;

        while let Some(n) = cur {
            if &n.key == key {
                existence = true;
                last = Some(n);
                break;
            }
            // push (parent.e.k, parent.mh)
            prefix.push((n.key.clone(), n.hash.clone()));
            if key < &n.key {
                cur = n.left.as_ref();
            } else {
                cur = n.right.as_ref();
            }
        }

        let (left_h, right_h, non_ex_key) = if existence {
            let ln = last
                .unwrap()
                .left
                .as_ref()
                .map(|x| x.hash.clone())
                .unwrap_or_default();
            let rn = last
                .unwrap()
                .right
                .as_ref()
                .map(|x| x.hash.clone())
                .unwrap_or_default();
            (ln, rn, None)
        } else {
            // non-existence: use the last traversed node as witness key
            let witness = prefix.last().map(|(k, _)| k.clone());
            let (ln, rn) = match cur {
                Some(n) => (
                    n.left.as_ref().map(|x| x.hash.clone()).unwrap_or_default(),
                    n.right.as_ref().map(|x| x.hash.clone()).unwrap_or_default(),
                ),
                None => (Vec::new(), Vec::new()),
            };
            (ln, rn, witness)
        };

        let suffix = [left_h, right_h];

        Proof {
            prefix,
            suffix,
            existence,
            nonexistence_key: non_ex_key,
        }
    }
}

pub struct Proof<K> {
    pub prefix: Vec<(K, Hash)>,
    pub suffix: [Hash; 2],
    pub existence: bool,
    pub nonexistence_key: Option<K>,
}

fn find_priority<K: AsRef<[u8]>>(key: &K) -> Priority {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(key.as_ref());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    i128::from_be_bytes(bytes) as i128
}
