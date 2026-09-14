use super::{Package, TypedFunc};
use crate::Verdict;

pub type TestFn = fn() -> Verdict<(), ()>;

pub struct TestCase {
    name: String,
    func: TypedFunc<TestFn>,
}

impl TestCase {
    pub fn new(name: String, func: TypedFunc<TestFn>) -> Self {
        Self { name, func }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TestCase {
    pub fn run(&self) -> Result<(), ()> {
        match self.func.call() {
            Verdict::Accept(()) => Ok(()),
            Verdict::Reject(()) => Err(()),
        }
    }
}

impl Package {
    pub fn tests(&mut self) -> impl Iterator<Item = TestCase> + use<'_> {
        let tests: Vec<_> = self
            .functions
            .keys()
            .filter(|x| {
                x.rsplit_once('.')
                    .map_or(x.as_ref(), |x| x.1)
                    .starts_with("test#")
            })
            .map(Clone::clone)
            .collect();

        tests.into_iter().map(|name| {
            let func = self.func::<TestFn>(name.strip_prefix("pkg.").unwrap());
            TestCase::new(name.replace("test#", ""), func.unwrap())
        })
    }

    #[allow(clippy::result_unit_err)]
    pub fn run_tests(&mut self) -> Result<(), ()> {
        let tests: Vec<_> = self.tests().collect();

        let total = tests.len();
        let total_width = total.to_string().len();
        let mut successes = 0;
        let mut failures = 0;

        for (n, test) in tests.into_iter().enumerate() {
            let n = n + 1;
            let test_display = test.name();
            print!("Test {n:>total_width$} / {total}: {test_display}... ");

            if test.run() == Ok(()) {
                successes += 1;
                println!("\x1B[92mok\x1B[m");
            } else {
                failures += 1;
                println!("\x1B[91mfail\x1B[m");
            }
        }
        println!("Ran {total} tests, {successes} succeeded, {failures} failed");

        if failures == 0 { Ok(()) } else { Err(()) }
    }
}
