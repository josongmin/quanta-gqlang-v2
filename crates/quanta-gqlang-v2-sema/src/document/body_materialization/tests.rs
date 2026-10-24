//! Request body allocation bounds.

use super::{
    request_exact_vec_upper_bytes_v3, reserve_scalar_body_vec_v3, ExpressionEffectsV2,
    OutputColumnContractV2, SourceQualifiedNodeIdV2, TypedExprV2, TypedRelationExprV2,
};

#[test]
fn admitted_scalar_vec_capacity_fits_prebirth_limit() {
    fn check<T>(items: usize) {
        let mut storage: Vec<T> = Vec::new();
        assert!(storage.try_reserve_exact(items).is_ok());
        let actual = storage.capacity() * std::mem::size_of::<T>();
        assert!(request_exact_vec_upper_bytes_v3::<T>(items).is_some_and(|bound| bound >= actual));
    }
    for items in [1, 9] {
        check::<(SourceQualifiedNodeIdV2, TypedExprV2)>(items * 2);
        check::<OutputColumnContractV2>(items);
        check::<SourceQualifiedNodeIdV2>(items);
        check::<ExpressionEffectsV2>(items);
    }
}

#[test]
fn admitted_filter_vec_capacity_fits_prebirth_limit() {
    for fields in [1_usize, 3, 9] {
        let expressions =
            reserve_scalar_body_vec_v3::<(SourceQualifiedNodeIdV2, TypedExprV2)>(fields * 2 + 4);
        let inputs = reserve_scalar_body_vec_v3::<TypedRelationExprV2>(1);
        let columns = reserve_scalar_body_vec_v3::<OutputColumnContractV2>(fields);
        assert!(expressions.as_ref().is_ok_and(|values| {
            values.capacity() * std::mem::size_of::<(SourceQualifiedNodeIdV2, TypedExprV2)>()
                <= request_exact_vec_upper_bytes_v3::<(SourceQualifiedNodeIdV2, TypedExprV2)>(
                    fields * 2 + 4,
                )
                .unwrap_or(0)
        }));
        assert!(inputs.as_ref().is_ok_and(|values| {
            values.capacity() * std::mem::size_of::<TypedRelationExprV2>()
                <= request_exact_vec_upper_bytes_v3::<TypedRelationExprV2>(1).unwrap_or(0)
        }));
        assert!(columns.as_ref().is_ok_and(|values| {
            values.capacity() * std::mem::size_of::<OutputColumnContractV2>()
                <= request_exact_vec_upper_bytes_v3::<OutputColumnContractV2>(fields).unwrap_or(0)
        }));
    }
}

#[test]
fn admitted_scalar_name_copies_fit_source_byte_lengths() {
    // These are the names and authored source lengths used by the one and
    // nine column request fixtures. The storage bound counts each name
    // copy by its source byte length, so allocator capacity must not grow.
    for name in [
        "r",
        "id",
        "id0",
        "id1",
        "id2",
        "id3",
        "id4",
        "id5",
        "id6",
        "id7",
        "id8",
        "host",
        "records",
        "view active | select=from records r | project r.id as id",
    ] {
        let owned = name.to_owned();
        let cloned = owned.clone();
        assert!(owned.capacity() <= name.len());
        assert!(cloned.capacity() <= name.len());
    }
}
