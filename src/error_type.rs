#![allow(dead_code)]

macro_rules! define_error_type {
    ($error_type:ident; $($kind_identifier:ident : $error_message:expr),* ) => (
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub enum $error_type {
            $( $kind_identifier ),*
        }

        impl $error_type {
            pub fn message(&self) -> &'static str {
                match self {
                    $( &$error_type::$kind_identifier => &$error_message ),*
                }
            }
        }

        impl ::std::fmt::Display for $error_type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                f.write_str(self.message())?;
                Ok(())
            }
        }

        impl ::std::error::Error for $error_type {
            fn description(&self) -> &str {
                self.message()
            }
        }

            mod define_error_type_test {
                #[test]
                fn test_consistency() {
                    use std::error::Error;
                    $(
                        {
                            let error = super::$error_type::$kind_identifier;
                            assert_eq!(error.description(), format!("{}", error));
                        }
                    )*;
                }
            }


    )
    // ($error_type:ident; $error_kind:ident; $($kind_identifier:ident : $error_message:expr),* ; $( $field_name:ident : $field_type:ty ),* ) => (
    //     #[derive(Copy, Clone, Debug, PartialEq)]
    //     pub enum $error_kind {
    //         $( $kind_identifier ),*
    //     }

    //     #[derive(Copy, Clone, Debug, PartialEq)]
    //     pub struct $error_type {
    //         kind: $error_kind
    //     }

    //     impl $error_type {
    //         pub fn message(&self) -> &'static str {
    //             match self.kind {
    //                 $( $error_kind::$kind_identifier => &$error_message ),*
    //             }
    //         }
    //     }

    //     impl ::std::fmt::Display for $error_type {
    //         fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
    //             f.write_str(self.message())?;
    //             Ok(())
    //         }
    //     }

    //     impl ::std::error::Error for $error_type {
    //         fn description(&self) -> &str {
    //             self.message()
    //         }
    //     }

    // )

}

define_error_type![TestError;A: "an a", B: "a b"];