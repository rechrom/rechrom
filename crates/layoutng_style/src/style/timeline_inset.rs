use foundation::Length;

// cpp: layoutng_style/style/timeline_inset.h:14-33
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TimelineInset {
    start_: Length,
    end_: Length,
}

#[allow(non_snake_case)]
impl TimelineInset {
    // cpp: layoutng_style/style/timeline_inset.h:17-18
    pub fn new(start: &Length, end: &Length) -> Self {
        Self {
            start_: start.clone(),
            end_: end.clone(),
        }
    }

    // cpp: layoutng_style/style/timeline_inset.h:20-24
    pub fn GetStart(&self) -> &Length {
        &self.start_
    }
    pub fn GetEnd(&self) -> &Length {
        &self.end_
    }
}
