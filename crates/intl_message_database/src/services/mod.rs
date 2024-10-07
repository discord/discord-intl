pub mod export;
pub mod precompile;
pub mod types;
pub mod validator;

pub(crate) trait IntlService {
    type Result;

    fn run(&mut self) -> Self::Result;
}
