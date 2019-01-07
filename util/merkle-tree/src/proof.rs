use crate::hash::Merge;
use crate::tree::Tree;
use std::collections::VecDeque;

/// Merkle Proof can provide a proof for existence of one or more items.
/// Only sibling of the nodes along the path that form leaves to root,
/// excluding the nodes already in the path, should be included in the proof.
/// For example, if we want to show that [T0, T5] is in the list of 6 items,
/// only nodes [T4, T1, B3] should be included in the proof.
///
/// `tree nodes`: [B0, B1, B2, B3, B4, T0, T1, T2, T3, T4, T5]
/// `leaves`: [(0, T0), (5, T5)]
/// `lemmas`: [T4, T1, B3]
/// `leaves_count`: 6
pub struct Proof<M>
where
    M: Merge,
{
    /// a partial leaves collection keeps the items sorted based on index
    pub leaves: Vec<(usize, M::Item)>,
    /// non-calculable nodes, stored in descending order
    pub lemmas: Vec<M::Item>,
    /// total leaves count
    pub leaves_count: usize,
}

impl<M> Proof<M>
where
    M: Merge,
    <M as Merge>::Item: Clone + Default,
{
    /// Returns the root of the proof, or None if it is empty or lemmas are invalid
    pub fn root(&self) -> Option<M::Item> {
        if self.leaves_count == 0 {
            return None;
        }

        let mut queue = self
            .leaves
            .iter()
            .rev()
            .map(|(index, leaf)| (self.leaves_count + index - 1, leaf.clone()))
            .collect::<VecDeque<_>>();

        let mut lemmas_iter = self.lemmas.iter();

        while let Some((index, node)) = queue.pop_front() {
            if index == 0 {
                // ensure that all lemmas and leaves are consumed
                if lemmas_iter.next().is_none() && queue.is_empty() {
                    return Some(node);
                } else {
                    return None;
                }
            }

            if let Some(sibling) = match queue.front() {
                Some((front, _)) if *front == index.sibling() => {
                    queue.pop_front().map(|i| i.1.clone())
                }
                _ => lemmas_iter.next().cloned(),
            } {
                let parent_node = if index.is_left() {
                    M::merge(&node, &sibling)
                } else {
                    M::merge(&sibling, &node)
                };
                queue.push_back((index.parent(), parent_node));
            }
        }

        None
    }
}

impl<M> Tree<M>
where
    M: Merge,
    <M as Merge>::Item: Clone + Default,
{
    /// Returns the proof of the tree, or None if it is empty.
    /// Assumes that the `leaf_indexes` is sorted.
    pub fn get_proof(&self, leaf_indexes: &[usize]) -> Option<Proof<M>> {
        let leaves_count = (self.nodes.len() >> 1) + 1;

        if self.nodes.is_empty()
            || leaf_indexes.is_empty()
            || *leaf_indexes.last().unwrap() >= leaves_count
        {
            return None;
        }

        let leaves = leaf_indexes
            .iter()
            .map(|&index| (index, self.nodes[leaves_count + index - 1].clone()))
            .collect::<Vec<_>>();

        let mut lemmas = Vec::new();
        let mut queue = leaf_indexes
            .iter()
            .rev()
            .map(|index| leaves_count + index - 1)
            .collect::<VecDeque<_>>();
        while let Some(index) = queue.pop_front() {
            if index == 0 {
                break;
            }
            if Some(&index.sibling()) == queue.front() {
                queue.pop_front();
            } else {
                lemmas.push(self.nodes[index.sibling()].clone());
            }

            queue.push_back(index.parent());
        }

        Some(Proof {
            leaves,
            lemmas,
            leaves_count,
        })
    }
}

/// A helper trait for node index
trait NodeIndex {
    fn sibling(&self) -> usize;
    fn parent(&self) -> usize;
    fn is_left(&self) -> bool;
}

impl NodeIndex for usize {
    #[inline]
    fn sibling(&self) -> usize {
        debug_assert!(*self > 0);
        ((self + 1) ^ 1) - 1
    }

    #[inline]
    fn parent(&self) -> usize {
        debug_assert!(*self > 0);
        (self - 1) >> 1
    }

    #[inline]
    fn is_left(&self) -> bool {
        self & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Tree;
    use proptest::collection::vec;
    use proptest::num::i32;
    use proptest::prelude::*;
    use proptest::sample::subsequence;
    use proptest::{proptest, proptest_helper};
    struct DummyHash;

    impl Merge for DummyHash {
        type Item = i32;

        fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
            right.wrapping_sub(*left)
        }
    }

    #[test]
    fn empty() {
        let proof: Proof<DummyHash> = Proof {
            leaves: vec![],
            lemmas: vec![],
            leaves_count: 0,
        };

        assert_eq!(None, proof.root());
    }

    #[test]
    fn one() {
        let proof: Proof<DummyHash> = Proof {
            leaves: vec![(0, 1)],
            lemmas: vec![],
            leaves_count: 1,
        };

        assert_eq!(Some(1), proof.root());
    }

    #[test]
    fn extra_lemma() {
        let proof: Proof<DummyHash> = Proof {
            leaves: vec![(0, 1)],
            lemmas: vec![1],
            leaves_count: 1,
        };

        assert_eq!(None, proof.root());
    }

    #[test]
    fn missing_leaves() {
        let proof: Proof<DummyHash> = Proof {
            leaves: vec![(1, 1)],
            lemmas: vec![],
            leaves_count: 2,
        };

        assert_eq!(None, proof.root());
    }

    #[test]
    // [ 1,  0,  1,  2,  2,  2,  3,  5,  7, 11, 13]
    // [B0, B1, B2, B3, B4, T0, T1, T2, T3, T4, T5]
    // [(0, 2), (5, 13)]
    // [    T0,      T5]
    // [11,  3,  2]
    // [T4, T1, B3]
    fn two_of_six() {
        let proof: Proof<DummyHash> = Proof {
            leaves: vec![(0, 2), (5, 13)],
            lemmas: vec![11, 3, 2],
            leaves_count: 6,
        };

        assert_eq!(Some(1), proof.root());
    }

    #[test]
    fn build_proof() {
        let leaves = vec![2, 3, 5, 7, 11, 13];
        let tree = Tree::<DummyHash>::new(&leaves);
        let proof = tree.get_proof(&[0, 5]).unwrap();
        assert_eq!(vec![(0, 2), (5, 13)], proof.leaves);
        assert_eq!(vec![11, 3, 2], proof.lemmas);
        assert_eq!(Some(1), proof.root());
    }

    fn _tree_root_is_same_as_proof_root(leaves: &[i32], indexes: &[usize]) {
        let tree = Tree::<DummyHash>::new(leaves);
        let proof = tree.get_proof(indexes).unwrap();
        assert_eq!(Tree::<DummyHash>::build_root(leaves), proof.root());
    }

    proptest! {
        #[test]
        fn tree_root_is_same_as_proof_root(input in vec(i32::ANY,  2..1000)
            .prop_flat_map(|leaves| (Just(leaves.clone()), subsequence((0..leaves.len()).collect::<Vec<usize>>(), 1..leaves.len())))
        ) {
            _tree_root_is_same_as_proof_root(&input.0, &input.1);
        }
    }
}
