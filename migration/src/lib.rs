mod m20260509_000001_initial_schema;

pub struct Migration {
    pub name: &'static str,
    pub up: fn() -> Vec<String>,
    pub down: fn() -> Vec<String>,
}

#[must_use]
pub fn migrations() -> Vec<Migration> {
    vec![Migration {
        name: "20260509_000001_initial_schema",
        up: m20260509_000001_initial_schema::up,
        down: m20260509_000001_initial_schema::down,
    }]
}
