pub struct ItemDatabase {}

pub struct Item;

pub trait Crafting {
    fn crafting(&self) -> Entity;
}

struct Food {
    hunger: f32,
    trist: f32,
}

pub enum RecipeFilter {
    Keep(Vec<String>),
    Consumed(Vec<String>),
}

fn make_portion(items: &mut [Item], result: &mut Item, player: &()) {
    let add_type = "Base.Pot";
    for item in items {
        // | "RicePan" | "PastaPan" | "PastaPot" | "RicePot" | "WaterPotRice"
        if matches!(item.name(), "PotOfSoup" | "PotOfSoupRecipe") {
            result.base_hunger = item.base_hunger / 4.0;
            result.hung_change = item.hung_change / 4.0;
            result.thirst_change = item.thirst_change / 4.0;

            result.boredom_change = item.boredom_change / 4.0;
            result.unhappy_change = item.unhappy_change / 4.0;
            result.carbohydrates = item.carbohydrates / 4.0;
            result.lipids = item.lipids / 4.0;
            result.proteins = item.proteins / 4.0;
            result.calories = item.calories / 4.0;
            result.tainted_water = item.tainted_water;
        }
    }

    player.inventory().add_item(add_type);
}

enum IngredientMode {
    Keep,
    Consume,
}

struct Ingredient {
    name: String,
    count: usize,
    mode: IngredientMode,
}

impl Ingredient {
    fn new(name: impl ToString, count: usize, mode: IngredientMode) -> Self {
        let name = name.to_string();
        Self { name, count, mode }
    }

    fn consume(name: impl ToString, count: usize) -> Self {
        Self::new(name, count, IngredientMode::Consume)
    }

    fn keep(name: impl ToString, count: usize) -> Self {
        Self::new(name, count, IngredientMode::Keep)
    }
}

pub struct Product {
    name: String,
    count: usize,
}

impl Product {
    fn new(name: impl ToString, count: usize) {
        let name = name.to_string();
        Self { name, count }
    }
}

pub struct Recipe {
    pub products: Vec<Product>,
    pub ingredients: Vec<Ingredient>,
    pub category: String,
    pub time: f32,
    // on_create: MakeBowlOfSoup2 | separate_food
}

fn example() {
    let mut recipes = HashMap::default();

    /*
    recipes[String::from("Make 2 Bowls of Soup")] = Recipe {
        product: vec![Product::consume("SoupBowl", 2)],
        category: String::from("Cooking"),
        ingredients: vec![
            Ingredient::consume("PotOfSoup", 1),
            Ingredient::consume("Bowl", 2),
        ],
        time: 80.0,
    };
    */

    recipes[String::from("Consume Bowl of Soup")] = Recipe {
        products: vec![Product::new("SoupBowl", 1), Product::new("Pot", 1)],
        category: String::from("Cooking"),
        ingredients: vec![
            Ingredient::keep("PotOfSoup", 1),
            Ingredient::consume("Bowl", 1),
        ],
        time: 80.0,
    };
}

fn consume_portion(dst: &mut (), part_result: &str, source_result: &str) {
    let container = dst.remove(igredients[1]);

    let amount = container.capacity.min(base.amount);

    let mut base = dst.find(igredients[0]);
    base.amount -= amount;

    let result = dst.add_item(products[0]);

    if base.amount <= 0.0 {
        dst.replace(ingredients[0], product[1]);
    }
}
