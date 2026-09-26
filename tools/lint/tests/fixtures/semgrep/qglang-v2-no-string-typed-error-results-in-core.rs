pub fn erased_error() -> Result<(), String> {
    Ok(())
}

pub async fn async_erased_error() -> Result<Vec<Option<u8>>, String> {
    Ok(vec![])
}

pub(crate) fn nested_erased_error() -> Result<Vec<Vec<Option<u8>>>, String> {
    Ok(vec![])
}
