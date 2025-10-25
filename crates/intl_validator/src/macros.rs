/// Create the boilerplate for a standard validation rule using `validate_cst`
/// as the entrypoint for validating a message.
macro_rules! cst_validation_rule {
    ($name:ident) => {
        pub struct $name {
            diagnostics: Vec<crate::diagnostic::ValueDiagnostic>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    diagnostics: vec![],
                }
            }
        }

        impl crate::validators::validator::Validator for $name {
            fn validate_cst(
                &mut self,
                message: &intl_database_core::MessageValue,
            ) -> Option<Vec<crate::diagnostic::ValueDiagnostic>> {
                message.parsed.visit_with(self);
                Some(self.diagnostics.clone())
            }
        }
    };
}

pub(crate) use cst_validation_rule;
