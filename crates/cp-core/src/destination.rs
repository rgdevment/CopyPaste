#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    pub pid: i32,
    pub bundle_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct Tracker {
    ours: i32,
    last_foreign: Option<Destination>,
}

impl Tracker {
    pub fn new(our_pid: i32) -> Self {
        Self {
            ours: our_pid,
            last_foreign: None,
        }
    }

    pub fn saw(&mut self, pid: i32, bundle_id: Option<&str>) {
        if pid == self.ours {
            return;
        }
        self.last_foreign = Some(Destination {
            pid,
            bundle_id: bundle_id.map(str::to_owned),
        });
    }

    pub fn destination(&self) -> Option<&Destination> {
        self.last_foreign.as_ref()
    }

    pub fn gone(&mut self, pid: i32) {
        if self.last_foreign.as_ref().is_some_and(|one| one.pid == pid) {
            self.last_foreign = None;
        }
    }
}

#[cfg(test)]
#[path = "destination_test.rs"]
mod tests;
