use file_reader::FileErr;


error_chain! {
     
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        LibParseError {
            description("unsuccessful parsing of file in RenLib format")
        }
        PosParseError {
            description("unsuccessful parsing of file in pos format")
        }
        NotSupported {
            description("File is not currently supported")
        }
    }
}
