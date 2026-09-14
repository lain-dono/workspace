use crate::env::Point;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ReportedError {
    point: Point,
    message: String,
}

#[derive(Default)]
pub struct ErrorReporting {
    reported_errors: Vec<ReportedError>,
    expected_errors: HashMap<Point, String>,
}

impl ErrorReporting {
    pub fn report_error(&mut self, point: Point, message: String) {
        self.reported_errors.push(ReportedError { point, message });
    }

    pub fn expect_error(&mut self, point: Point, message: &str) {
        let old_entry = self.expected_errors.insert(point, message.to_string());
        assert!(old_entry.is_none());
    }

    pub fn reconcile_errors(&mut self) -> Result<(), ReportedError> {
        while let Some(reported_error) = self.reported_errors.pop() {
            if let Some(expected_message) = self.expected_errors.remove(&reported_error.point) {
                if reported_error.message.contains(&expected_message) {
                    continue;
                }
            }
            return Err(reported_error);
        }

        if let Some(&point) = self.expected_errors.keys().next() {
            let message = String::from("no error reported on this point, but we expected one");
            return Err(ReportedError { point, message });
        }

        Ok(())
    }
}

impl Error for ReportedError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ReportedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}: {}", self.point, self.message)
    }
}
