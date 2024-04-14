#[macro_export]
macro_rules! not_found {
    ($result:expr, $expect:expr) => {{
        let error = $result.expect_err($expect);
        let kind = error.kind();
        match kind {
            ::minty::ErrorKind::NotFound => (),
            _ => {
                panic!(
                    "expected 'not found' error; got {kind:?} error: {error}"
                )
            }
        }
    }};
    ($result:expr, $entity:literal, $id:expr) => {{
        let expect = format!("{} with ID '{}' should not exist", $entity, $id);
        let error = $result.expect_err(&expect);
        let kind = error.kind();
        match kind {
            ::minty::ErrorKind::NotFound => {
                let expected =
                    format!("{} with ID '{}' not found", $entity, $id);
                let actual = error.to_string();
                assert_eq!(expected, actual);
            }
            _ => panic!(
                "expected '{} not found' error; got {kind:?} error: {error}",
                $entity
            ),
        }
    }};
}
