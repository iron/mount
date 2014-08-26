use std::collections::HashMap;

// Matches only literal routes
pub struct Matcher<V> {
    value: Option<V>,
    children: HashMap<String, Matcher<V>>
}

impl<V> Matcher<V> {
    pub fn new() -> Matcher<V> { Matcher { value: None, children: HashMap::new() } }
    pub fn insert(&mut self, key: Vec<String>, value: V) {
        key.move_iter().fold(self, |this, string| {
           this.children.find_or_insert(string, Matcher::new())
        }).value = Some(value);
    }

    pub fn find(&self, key: &[String]) -> Option<&V> {
        key.iter().fold(Some(self), |this, string| {
            this.and_then(|th| th.children.find(string))
        }).and_then(|x| x.value.as_ref())
    }
}

