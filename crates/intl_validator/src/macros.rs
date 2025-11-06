/// Create the boilerplate for a standard validation rule using `validate_cst`
/// as the entrypoint for validating a message.
macro_rules! cst_validation_rule {
    ($name:ident) => {
        pub struct $name {
            context: crate::validators::validator::ValidatorContext,
        }

        impl $name {
            pub fn new(locale: intl_database_core::KeySymbol) -> Self {
                Self {
                    context: crate::validators::validator::ValidatorContext::new(locale),
                }
            }
        }

        impl crate::validators::validator::Validator for $name {
            fn validate_cst(
                &mut self,
                message: &intl_database_core::MessageValue,
            ) -> Option<Vec<crate::diagnostic::ValueDiagnostic>> {
                message.parsed.visit_with(self);
                Some(self.context.diagnostics.clone())
            }
        }
    };
}

pub(crate) use cst_validation_rule;
