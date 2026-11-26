use super::*;

#[tokio::test]
async fn the_result_of_the_work_is_returned_as_it_is() {
    assert_eq!(blocking(|| Ok(7)).await.unwrap(), 7);
    assert!(matches!(blocking::<()>(|| Err(RekallError::conflict("busy"))).await, Err(RekallError::Conflict(_))));
}

#[tokio::test]
async fn work_that_panics_is_an_internal_error_and_not_a_panic() {
    let outcome = blocking::<()>(|| panic!("the job broke")).await;

    assert!(matches!(outcome, Err(RekallError::Internal(_))));
}
