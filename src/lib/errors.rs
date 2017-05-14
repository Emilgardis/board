use file_reader::FileErr;


error_chain! {
     
    foreign_links {
        Io(::std::io::Error);
        ParseInt(::std::num::ParseIntError);
    }

    errors {
        LibParseError {
            description("unsuccessful parsing of file in RenLib format")
        }
        PosParseError {
            description("unsuccessful parsing of file in pos format")
        }
        VersionNotSupported(majv: u8, minv: u8) {
            display("Version not supported")
            description("Version {}.{} is not supported")
        }
        NotSupported {
            description("File is not currently supported")
        }
        MoveIndexParseError {
            description("Couldn't parse MoveIndex string")
        }
    }
}
