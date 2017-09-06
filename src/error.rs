
error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }
    links {
    }

    foreign_links {
    }

    errors {
        InitFailed {
            description("failed to init the steamworks API"),
        }
    }
}
