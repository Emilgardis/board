use board_logic::{BoardMarker, Stone, Point};
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::rc::Rc;
use std::rc::Weak; // This should probably be used, not sure how.

pub fn ref_filter_map<
    T: ?Sized,
    U: ?Sized,
    F: FnOnce(&T) -> Option<&U>
>(orig: Ref<T>, f: F) -> Option<Ref<U>> {
    f(&orig)
        .map(|new| new as *const U)
        .map(|raw| Ref::map(orig, |_| unsafe { &*raw }))
}

pub fn ref_mut_filter_map<
    T: ?Sized,
    U: ?Sized,
    F: FnOnce(&mut T) -> Option<&mut U>
>(mut orig: RefMut<T>, f: F) -> Option<RefMut<U>> {
    f(&mut orig)
        .map(|new| new as *mut U)
        .map(|raw| RefMut::map(orig, |_| unsafe { &mut *raw }))
}

#[derive(Debug)]
struct Node<'a> {
    children: RefCell<Vec<Rc<Node<'a>>>>, // Should it be RefCell instead of Rc?
    parent: Option<Weak<Node<'a>>>, // Weak instead of Rc.
    mark: RefCell<Option<BoardMarker>>, // Should this be a Cell? Or should it own the BoardMarker?
}

impl<'a> Node<'a> {
    fn new_root() -> Rc<Node<'a>> {
        Rc::new(Node {
            children: RefCell::new(Vec::new()),
            parent: None,
            mark: RefCell::new(None),
        }) }
    fn new(mark: BoardMarker, parent: Weak<Node<'a>>) -> Node<'a> {
        Node {
            children: RefCell::new(Vec::new()),
            parent: Some(parent),
            mark: RefCell::new(Some(mark)),
        }
    }
    // fn find_child(mark: Pos) -> Result<Rc<Node>, bool> ?
    fn add_child(&self, mark: BoardMarker, root: Rc<Node<'a>>) -> Result<Ref<Rc<Node<'a>>>, bool> {

        self.children.borrow_mut().push(Rc::new(Node::new(mark, Rc::downgrade(&root))));
        Ok(self.get_nth(self.children.borrow().len()-1).unwrap())
    }
    
    fn is_root(&self) -> bool {
        self.parent.is_none()
    }
    
    fn get_nth(&self, index: usize) -> Result<Ref<Rc<Node<'a>>>, bool>{
        let temp: Ref<Vec<Rc<Node>>> = self.children.borrow(); // children: RefCell<Vec<Rc<Node<'a>>>>
        match ref_filter_map(temp, |children| children.get(index)) {
            Some(val) => Ok(val),
            None => Err(true),
        }
    }
    
    //fn get_parent(&self) -> Result<Weak<Node<'a>>, bool>{
    //    Ok(self.parent.unwrap())
    //}
    
}

#[test]
fn check_if_node_kinda_works() {
    let mut root: Rc<Node> = Node::new_root();
    println!("Is root: {}", root.is_root());
    let child1: Ref<Rc<Node>> = root.add_child(BoardMarker { point: Point { x: 0, y: 0}, color: Stone::Black}, root.clone()).unwrap();
    // let child1: Ref<Rc<Node>> =  root.get_nth(0).unwrap();
    println!("{:?}", child1);
    {let mut child1mark: RefMut<Option<BoardMarker>> = child1.mark.borrow_mut();
    *child1mark = Some(BoardMarker { point: Point { x: 14, y: 14}, color: Stone::Black});}
    println!("{:?}", root.get_nth(0));
    //println!("{:?}", root.get_parent());
}
