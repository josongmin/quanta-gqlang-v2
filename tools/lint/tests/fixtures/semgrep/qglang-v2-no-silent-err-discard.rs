pub fn discard(result: Result<(), ()>) {
    match result {
        Ok(()) => {}
        Err(_) => {}
    }
}
