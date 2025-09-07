use crate::utils::calculate_merkle_hash;
use parking_lot::RwLock;

mod utils;

pub type Key = [u8; 32];
pub type Priority = i128;
pub type Hash = Vec<u8>;
pub type Value = Vec<u8>;

pub trait Hasher {
    fn hash(data: &[u8]) -> Hash;
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub key: Key,
    pub priority: Priority,
    pub value: Value,
    pub hash: Hash,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for TreeNode {}

pub struct CartesianMerkleTree {
    root: RwLock<Option<Box<TreeNode>>>,
}

impl CartesianMerkleTree {
    pub fn new() -> Self {
        Self {
            root: RwLock::new(None),
        }
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        let cur = self.root.read();
        let mut cur = cur.as_ref();
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

    pub fn insert(&self, key: Key, value: Value) {
        let priority = find_priority(&key);
        let mut root = self.root.write();
        *root = Self::insert_recursive(root.take(), key, value, priority);
    }

    fn insert_recursive(
        node: Option<Box<TreeNode>>,
        key: Key,
        value: Value,
        priority: Priority,
    ) -> Option<Box<TreeNode>> {
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
            let (left_hash, right_hash) = rayon::join(
                || {
                    new_node
                        .left
                        .as_ref()
                        .map(|n| n.hash.clone())
                        .unwrap_or_default()
                },
                || {
                    new_node
                        .right
                        .as_ref()
                        .map(|n| n.hash.clone())
                        .unwrap_or_default()
                },
            );
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

        let (left_hash, right_hash) = rayon::join(
            || {
                current_node
                    .left
                    .as_ref()
                    .map(|n| n.hash.clone())
                    .unwrap_or_default()
            },
            || {
                current_node
                    .right
                    .as_ref()
                    .map(|n| n.hash.clone())
                    .unwrap_or_default()
            },
        );
        current_node.hash = calculate_merkle_hash(&current_node.key, &left_hash, &right_hash);

        Some(current_node)
    }

    fn split(
        node: &mut Box<TreeNode>,
        key: &Key,
        left: &mut Option<Box<TreeNode>>,
        right: &mut Option<Box<TreeNode>>,
    ) {
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
    pub fn remove(&self, key: &Key) {
        let mut root = self.root.write();
        *root = Self::remove_recursive(root.take(), key);
    }

    fn remove_recursive(node: Option<Box<TreeNode>>, key: &Key) -> Option<Box<TreeNode>> {
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
            let (left_hash, right_hash) = rayon::join(
                || {
                    current_node
                        .left
                        .as_ref()
                        .map(|n| n.hash.clone())
                        .unwrap_or_default()
                },
                || {
                    current_node
                        .right
                        .as_ref()
                        .map(|n| n.hash.clone())
                        .unwrap_or_default()
                },
            );
            current_node.hash = calculate_merkle_hash(&current_node.key, &left_hash, &right_hash);
            return Some(current_node);
        }
        None
    }

    fn heapify(node: Box<TreeNode>) -> Option<Box<TreeNode>> {
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

    pub fn generate_proof(&self, key: &Key) -> Proof {
        let mut prefix: Vec<(Key, Hash)> = Vec::new();
        let cur = self.root.read();
        let mut cur = cur.as_ref();
        let mut last: Option<&TreeNode> = None;
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
    pub fn verify_proof(proof: Proof, key: Key, root_hash: Hash) -> bool
where {
        let mut acc = Vec::new();
        if proof.existence {
            acc = calculate_merkle_hash(&key, proof.suffix[0].as_ref(), proof.suffix[1].as_ref());
        } else {
            acc = calculate_merkle_hash(
                &proof.nonexistence_key.unwrap(),
                proof.suffix[0].as_ref(),
                proof.suffix[1].as_ref(),
            )
        }

        for (k, mh) in proof.prefix {
            acc = calculate_merkle_hash(&k, &acc, &mh)
        }

        acc == root_hash
    }
}

pub struct Proof {
    pub prefix: Vec<(Key, Hash)>,
    pub suffix: [Hash; 2],
    pub existence: bool,
    pub nonexistence_key: Option<Key>,
}

fn find_priority(key: &Key) -> Priority {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(key.as_ref());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    i128::from_be_bytes(bytes) as i128
}
