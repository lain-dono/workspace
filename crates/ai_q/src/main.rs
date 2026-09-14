use std::collections::{BinaryHeap, HashSet};

pub mod test;

fn main() {
    println!("Hello, world!");

    let drink = Interaction::new(
        "Drink",
        vec![is_anywhere, is_sit_or_stand, is_holding_drink],
    );
    let sit = Interaction::new("Sit", vec![]);
    let watch_tv = Interaction::new("Watch TV", vec![]);
    let read_book = Interaction::new("Read Book", vec![]);
}

struct Interaction {
    name: String,
    constraints: Vec<Constraint>,
}

impl Interaction {
    fn new(name: impl Into<String>, constraints: Vec<Constraint>) -> Self {
        let name = name.into();
        Self { name, constraints }
    }
}

struct Priority<T>(u8, T);

impl<T> PartialEq for Priority<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> PartialOrd for Priority<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for Priority<T> {}
impl<T> Ord for Priority<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

struct State {
    queue: BinaryHeap<Priority<Interaction>>,
    // next
    active: HashSet<Interaction>,
    carry: [Item; 2],
    posture: Posture,
}

//allow
//block

enum Posture {
    Sit,
    Stand,
}

#[derive(PartialEq, Eq)]
enum Item {
    Drink,
    TvRemote,
}

type Constraint = fn(&State) -> bool;

fn is_anywhere(s: &State) -> bool {
    true
}

fn is_holding_drink(s: &State) -> bool {
    s.carry.iter().any(|x| matches!(x, Item::Drink))
}

fn is_sit_or_stand(s: &State) -> bool {
    matches!(s.posture, Posture::Sit | Posture::Stand)
}

struct ConstraintSet {
    location: Where,
    posture: Option<How>,
    hands: Hands,
}

impl ConstraintSet {
    fn check(&self, s: &State) -> bool {
        let hands = self.hands.check(s);
        hands
    }
}

enum Where {
    Anywhere,
    OnSeat,
}

enum How {
    Sit,
    SitOrStand,
}

enum Hands {
    Any,
    Hold(Item),
}

impl Hands {
    fn check(&self, s: &State) -> bool {
        match self {
            Self::Any => true,
            Self::Hold(item) => s.carry.contains(item),
        }
    }
}

trait ConstraintImpl: Sized {
    fn interaction(&self, other: &Self) -> Option<Self>;
    fn combination(&self, other: &Self) -> Option<Self>;
}

struct TimeMark;

struct Task {
    priority: u8,
    duration: f32,
    progress: f32,
    start_at: TimeMark,

    name: String,
    incompatible_with: Vec<String>,
}

impl Task {
    /// Check if this task is compatible with another
    fn can_run_with(&self, other: &Self) -> bool {
        !self.incompatible_with.contains(&other.name)
    }
    fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Task {}
impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

mod iii {
    struct Subject;
    struct Target;
    struct Context;

    enum TestResult {
        Accept,
        Reject(String),
        Hidden,
    }

    trait Test {
        fn test(&self, subject: &Subject, target: &Target, context: &Context) -> TestResult;
    }

    trait Interaction: Test {}

    struct Example;

    impl Test for Example {
        fn test(&self, _: &Subject, _: &Target, _: &Context) -> TestResult {
            let result = 1 + 1;
            match result {
                // Interaction will be hidden completely.
                3 => TestResult::Hidden,
                // Interaction will be displayed, but disabled,
                // it will also have a tooltip that displays on hover with the text "Test Tooltip"
                2 => TestResult::Reject(String::from("Test Tooltip")),
                // Interaction will display and be enabled.
                _ => TestResult::Accept,
            }
        }
    }
}
