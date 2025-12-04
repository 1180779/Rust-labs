use std::borrow::Cow;
use std::cell::{Cell, LazyCell, OnceCell, RefCell};
use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

struct AustroHungarianGreeter {
    greeting: Cell<i8>,
    greetings_invoked: Cell<u32>,
}

impl AustroHungarianGreeter {
    fn new() -> Self {
        AustroHungarianGreeter {
            greeting: Cell::new(0),
            greetings_invoked: Cell::new(0),
        }
    }
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
    for _i in 0..xs.len() {
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
        let first_rc: Rc<RefCell<Vertex>> = Rc::new(RefCell::new(Vertex::new()));
        let mut current_rc = Rc::clone(&first_rc);

        for i in 1..n {
            let new_neighbour_rc = current_rc.borrow_mut().create_neighbour();
            new_neighbour_rc.borrow_mut().data = i as i32;
            current_rc = new_neighbour_rc;
        }

        current_rc.borrow_mut().link_to(&first_rc);
        first_rc
    }
}

fn main() {
    // AustroHungarianGreeter
    let greeter = AustroHungarianGreeter::new();
    greeter.greet();

    // SharedFile
    let path = Path::new("test.txt");
    let mut f = fs::File::create(path).unwrap();
    f.write_all(b"Hello, world!").unwrap();
    let file = SharedFile::new(path.into());
    println!("file content: {}", file.get());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_greeter() {
        let greeter = AustroHungarianGreeter::new();
        assert_eq!(greeter.greet(), "Es lebe der Kaiser!");
        assert_eq!(greeter.greet(), "Möge uns der Kaiser schützen!");
        assert_eq!(greeter.greet(), "Éljen Ferenc József császár!");
        assert_eq!(greeter.greet(), "Es lebe der Kaiser!");
        assert_eq!(greeter.greetings_invoked.get(), 4);
    }

    #[test]
    fn test_heap_or_stack() {
        let mut stack_val = HeapOrStack::Stack(42);
        let mut heap_val = HeapOrStack::Heap(Box::new(42));

        assert_eq!(*stack_val, 42);
        assert_eq!(*heap_val, 42);

        *stack_val = 43;
        *heap_val = 43;

        assert_eq!(*stack_val, 43);
        assert_eq!(*heap_val, 43);
    }

    #[test]
    fn test_canon_head() {
        let dq = VecDeque::from(vec![2, 4, 6, 1, 8]);
        let cow = canon_head(&dq).unwrap();
        assert_eq!(*cow, VecDeque::from(vec![1, 8, 2, 4, 6]));
        assert!(matches!(cow, Cow::Owned(_)));

        let dq2 = VecDeque::from(vec![1, 2, 3, 4, 5]);
        let cow2 = canon_head(&dq2).unwrap();
        assert_eq!(*cow2, dq2);
        assert!(matches!(cow2, Cow::Borrowed(_)));

        let dq3 = VecDeque::from(vec![2, 4, 6, 8]);
        let cow3 = canon_head(&dq3).unwrap();
        assert_eq!(*cow3, VecDeque::from(vec![2, 4, 6, 8]));

        let dq4 = VecDeque::new();
        assert!(canon_head(&dq4).unwrap().is_empty());
    }

    #[test]
    fn test_cached_file() {
        let file = CachedFile::new();
        assert_eq!(file.try_get(), None);

        let path = Path::new("test.txt");
        let mut f = fs::File::create(path).unwrap();
        f.write_all(b"Hello, world!").unwrap();

        assert_eq!(file.try_get(), None);
        assert_eq!(file.get(path), "Hello, world!");
        assert_eq!(file.try_get(), Some("Hello, world!"));
        assert_eq!(file.get(path), "Hello, world!");
    }

    #[test]
    fn test_shared_file() {
        let path = PathBuf::from("shared_test.txt");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(b"Shared content").unwrap();

        let file1 = SharedFile::new(path.clone());
        let file2 = file1.clone();

        assert_eq!(file1.get(), "Shared content");
        assert_eq!(file2.get(), "Shared content");

        assert_eq!(file1.get().as_ptr(), file2.get().as_ptr());

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_vertex() {
        let mut v1 = Vertex::new();
        v1.data = 1;
        let v2 = v1.create_neighbour();
        v2.borrow_mut().data = 2;
        let v3 = v1.create_neighbour();
        v3.borrow_mut().data = 3;

        v2.borrow_mut().link_to(&v3);

        assert_eq!(v1.out_edges_owned.len(), 2);
        assert_eq!(v1.out_edges.len(), 0);
        assert_eq!(v2.borrow().out_edges_owned.len(), 0);
        assert_eq!(v2.borrow().out_edges.len(), 1);

        let neighbours = v1.all_neighbours();
        assert_eq!(neighbours.len(), 2);
    }

    #[test]
    fn test_vertex_cycle(){
        let cycle_head = Vertex::cycle(5);
        let mut current = Rc::downgrade(&cycle_head);
        for i in 0..5 {
            assert_eq!(current.upgrade().unwrap().borrow().data, i);
            let next_strong = current.upgrade().unwrap().borrow().all_neighbours()[0].clone();
            current = next_strong;
        }

        assert!(Weak::ptr_eq(&Rc::downgrade(&current.upgrade().unwrap()), &Rc::downgrade(&cycle_head)));
    }
    
    #[test]
    fn test_vertex_cycle_memory() {
        let n = 10;
        let head_rc = Vertex::cycle(n);
        let head_weak = Rc::downgrade(&head_rc);

        drop(head_rc);

        assert!(head_weak.upgrade().is_none(), "Cycle was not properly deallocated!");
    }
}
