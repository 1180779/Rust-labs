use std::borrow::Cow;
use std::cell::{Cell, LazyCell, OnceCell, RefCell, RefMut};
use std::collections::VecDeque;
use std::fs;
use std::io::read_to_string;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

struct AustroHungarianGreeter {
    greeting: Cell<i8>,
    greetings_invoked: Cell<u32>,
}

impl AustroHungarianGreeter {
    fn greet(&self) -> &str {
        self.greetings_invoked.set(self.greetings_invoked.get() + 1);
        match self.greeting.get() {
            0 => {
                self.greeting.set(1);
                "Es lebe der Kaiser!"
            }
            1 => {
                self.greeting.set(2);
                "Möge uns der Kaiser schützen!"
            }
            _ => {
                self.greeting.set(0);
                "Éljen Ferenc József császár!"
            }
        }
    }
}

impl Drop for AustroHungarianGreeter {
    fn drop(&mut self) {
        println!("Ich habe {} mal gegrüßt", self.greetings_invoked.get());
    }
}

pub enum HeapOrStack<T> {
    Stack(T),
    Heap(Box<T>),
}

impl<T> Deref for HeapOrStack<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            HeapOrStack::Stack(value) => value,
            HeapOrStack::Heap(boxed_value) => boxed_value.as_ref(),
        }
    }
}

impl<T> DerefMut for HeapOrStack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            HeapOrStack::Stack(value) => value,
            HeapOrStack::Heap(boxed_value) => boxed_value.as_mut(),
        }
    }
}

pub fn canon_head<'a>(xs: &'a VecDeque<i32>) -> Option<Cow<'a, VecDeque<i32>>> {
    let mut cow = Cow::Borrowed(xs);
    for i in 0..xs.len() {
        let front = cow.front();
        let front = front?;
        if front % 2 == 1 {
            return Some(cow);
        }
        let copy = *front;

        // Rotate front to back
        cow.to_mut().pop_front();
        cow.to_mut().push_back(copy);
    }
    Some(cow)
}

pub struct CachedFile {
    cache: OnceCell<String>,
}

impl Default for CachedFile {
    fn default() -> Self {
        Self::new()
    }
}

impl CachedFile {
    pub fn new() -> Self {
        CachedFile {
            cache: OnceCell::new(),
        }
    }
    /// Jeżeli cache jest pusty, wczytaj plik z `path` i zapisz w `cache`.
    /// Jeżeli cache jest już ustawiony, zwróć jego zawartość.
    pub fn get(&self, path: &Path) -> &str {
        let Some(cache) = self.cache.get() else {
            let file = fs::read_to_string(path);
            let Ok(contents) = file else {
                return "";
            };
            let res = self.cache.set(contents);
            if let Err(_e) = res {
                return "";
            }

            let res = self.cache.get();
            return match res {
                None => "",
                Some(s) => s.as_str(),
            };
        };
        cache
    }
    /// Jeżeli cache jest pusty, zwróć `None`
    /// Jeżeli cache jest już ustawiony, zwróć jego zawartość.
    pub fn try_get(&self) -> Option<&str> {
        match self.cache.get() {
            None => None,
            Some(s) => Some(s.as_str()),
        }
    }
    // Użyj std::fs::read_to_string oraz OnceCell.
}

#[derive(Clone)]
pub struct SharedFile {
    file: Rc<LazyCell<String, Box<dyn FnOnce() -> String>>>,
}

impl SharedFile {
    /// tworzy obiekt, który leniwie wczyta zawartość pliku przy pierwszym dostępie.
    fn new(path: PathBuf) -> Self {
        SharedFile {
            file: Rc::new(LazyCell::new(Box::new(|| {
                fs::read_to_string(path).unwrap()
            }))),
        }
    }

    /// zwraca referencję do treści pliku; wiele klonów SharedFile współdzieli ten sam bufor.
    fn get(&self) -> &str {
        self.file.as_str()
    }
}

pub struct Vertex {
    pub out_edges_owned: Vec<Rc<RefCell<Vertex>>>,
    pub out_edges: Vec<Weak<RefCell<Vertex>>>,
    pub data: i32,
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new()
    }
}

impl Vertex {
    // Tworzy pusty wierzchołek (data = 0, puste wektory).
    pub fn new() -> Self {
        Vertex {
            out_edges_owned: Vec::new(),
            out_edges: Vec::new(),
            data: 0,
        }
    }

    /// Tworzy nowego sąsiada z data = 0,
    /// dodaje go do `out_edges_owned` i zwraca `Rc` na niego.
    pub fn create_neighbour(&mut self) -> Rc<RefCell<Vertex>> {
        let neighbour = Rc::new(RefCell::new(Vertex::new()));
        self.out_edges_owned.push(neighbour.clone());
        neighbour
    }

    /// Dodaje krawędź do istniejącego wierzchołka jako słaba referencja (`Weak`).
    pub fn link_to(&mut self, other: &Rc<RefCell<Vertex>>) {
        self.out_edges.push(Rc::downgrade(other));
    }

    /// Zwraca wszystkich sąsiadów w postaci `Weak` (zarówno owned, jak i borrowed).
    pub fn all_neighbours(&self) -> Vec<Weak<RefCell<Vertex>>> {
        self.out_edges_owned
            .iter()
            .map(Rc::downgrade)
            .chain(self.out_edges.iter().cloned())
            .collect()
    }

    /// Buduje cykl długości `n`: v0 -> v1 -> ... -> v{n-1} -> v0
    /// z danymi odpowiednio 0, 1, ..., n - 1
    /// Zadbaj o to, aby cykl nie powodował wycieków pamięci!
    /// (odpowiednio używaj `create_neighbour` i `link_to`)
    pub fn cycle(n: usize) -> Rc<RefCell<Vertex>> {
        let mut first_rc: Rc<RefCell<Vertex>> = Rc::new(RefCell::new(Vertex::new()));
        let mut last = first_rc.borrow_mut();
        for i in 0..n {
            let neighbour = last.create_neighbour();
            last = neighbour.borrow_mut();
        }
        last.link_to(&first_rc);
        first_rc
    }
}

fn main() {
    println!("Hello, world!");
}
