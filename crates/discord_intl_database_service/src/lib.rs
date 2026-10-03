pub trait IntlDatabaseService {
    type Result;

    fn run(&mut self) -> Self::Result;
}
