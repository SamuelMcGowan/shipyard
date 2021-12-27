use super::{Modification, Tracking};
use crate::view::ViewMut;
use crate::{seal::Sealed, Component, EntityId, SparseSet, SparseSetDrain};

impl Sealed for Modification {}

impl<T: Component<Tracking = Modification>> Tracking<T> for Modification {
    #[inline]
    fn track_modification() -> bool {
        true
    }

    fn is_modified(
        sparse_set: &SparseSet<T, Self>,
        entity: EntityId,
        last: u32,
        current: u32,
    ) -> bool {
        if let Some(dense) = sparse_set.index_of(entity) {
            super::is_track_within_bounds(sparse_set.modification_data[dense], last, current)
        } else {
            false
        }
    }

    #[inline]
    fn remove(sparse_set: &mut SparseSet<T, Self>, entity: EntityId) -> Option<T> {
        sparse_set.actual_remove(entity)
    }

    #[inline]
    fn delete(sparse_set: &mut SparseSet<T, Self>, entity: EntityId) -> bool {
        sparse_set.actual_remove(entity).is_some()
    }

    fn clear(sparse_set: &mut SparseSet<T, Self>) {
        for &id in &sparse_set.dense {
            unsafe {
                *sparse_set.sparse.get_mut_unchecked(id) = EntityId::dead();
            }
        }

        sparse_set.dense.clear();
        sparse_set.data.clear();
    }

    #[track_caller]
    #[inline]
    fn apply<R, F: FnOnce(&mut T, &T) -> R>(
        sparse_set: &mut ViewMut<'_, T, Self>,
        a: EntityId,
        b: EntityId,
        f: F,
    ) -> R {
        let a_index = sparse_set.index_of(a).unwrap_or_else(move || {
            panic!(
                "Entity {:?} does not have any component in this storage.",
                a
            )
        });
        let b_index = sparse_set.index_of(b).unwrap_or_else(move || {
            panic!(
                "Entity {:?} does not have any component in this storage.",
                b
            )
        });

        if a_index != b_index {
            {
                unsafe {
                    *sparse_set.modification_data.get_unchecked_mut(a_index) = sparse_set.current;
                }
            }

            let a = unsafe { &mut *sparse_set.data.as_mut_ptr().add(a_index) };
            let b = unsafe { &*sparse_set.data.as_mut_ptr().add(b_index) };

            f(a, b)
        } else {
            panic!("Cannot use apply with identical components.");
        }
    }

    #[track_caller]
    #[inline]
    fn apply_mut<R, F: FnOnce(&mut T, &mut T) -> R>(
        sparse_set: &mut ViewMut<'_, T, Self>,
        a: EntityId,
        b: EntityId,
        f: F,
    ) -> R {
        let a_index = sparse_set.index_of(a).unwrap_or_else(move || {
            panic!(
                "Entity {:?} does not have any component in this storage.",
                a
            )
        });
        let b_index = sparse_set.index_of(b).unwrap_or_else(move || {
            panic!(
                "Entity {:?} does not have any component in this storage.",
                b
            )
        });

        if a_index != b_index {
            unsafe {
                *sparse_set.modification_data.get_unchecked_mut(a_index) = sparse_set.current;
                *sparse_set.modification_data.get_unchecked_mut(b_index) = sparse_set.current;
            }

            let a = unsafe { &mut *sparse_set.data.as_mut_ptr().add(a_index) };
            let b = unsafe { &mut *sparse_set.data.as_mut_ptr().add(b_index) };

            f(a, b)
        } else {
            panic!("Cannot use apply with identical components.");
        }
    }

    fn drain(sparse_set: &mut SparseSet<T, Self>) -> SparseSetDrain<'_, T> {
        for id in &sparse_set.dense {
            // SAFE ids from sparse_set.dense are always valid
            unsafe {
                *sparse_set.sparse.get_mut_unchecked(*id) = EntityId::dead();
            }
        }

        let dense_ptr = sparse_set.dense.as_ptr();
        let dense_len = sparse_set.dense.len();

        unsafe {
            sparse_set.dense.set_len(0);
        }

        SparseSetDrain {
            dense_ptr,
            dense_len,
            data: sparse_set.data.drain(..),
        }
    }
}
