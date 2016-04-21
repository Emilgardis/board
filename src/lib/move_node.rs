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
    children: Vec<RefCell<Rc<Node<'a>>>>, // Should it be RefCell instead of Rc?
    parent: Option<Weak<Node<'a>>>, // Weak instead of Rc.
    mark: Option<BoardMarker>, // Should this be a Cell? Or should it own the BoardMarker?
}

impl<'a> Node<'a> {
    fn new_root() -> RefCell<Rc<Node<'a>>> {
        RefCell::new(Rc::new(Node {
            children: Vec::new(),
            parent: None,
            mark: None,
        })) }
    fn new(mark: BoardMarker, parent: Weak<Node<'a>>) -> Node<'a> {
        Node {
            children: Vec::new(),
            parent: Some(parent),
            mark: Some(mark),
        }
    }
    // fn find_child(mark: Pos) -> Result<Rc<Node>, bool> ?
    fn add_child(&mut self, mark: BoardMarker, root: RefCell<Rc<Node<'a>>>) -> Result<RefCell<Rc<Node<'a>>>, bool> {
        {
            self.children.push(RefCell::new(Rc::new(Node::new(mark, Rc::downgrade(&root.borrow())))));
        }
        Ok(self.get_nth(&self.children.len()-1).unwrap())
    }
    
    fn is_root(&self) -> bool {
        self.parent.is_none()
    }
    
    fn get_nth(&self, index: usize) -> Result<RefCell<Rc<Node<'a>>>, bool>{
        match self.children.get(index) {
            Some(val) => Ok(val.clone()),
            None => Err(true),
        } // children: Vec<RefCell<Rc<Node<'a>>>>
    }
    
    //fn get_parent(&self) -> Result<Weak<Node<'a>>, bool>{
    //    Ok(self.parent.unwrap())
    //}
    
}

#[test]
fn check_if_node_kinda_works() {
    let mut root: RefCell<Rc<Node>> = Node::new_root();
    println!("Is root: {}", root.borrow().is_root());
    let mut child1: RefCell<Rc<Node>> = root.borrow_mut().add_child(BoardMarker { point: Point { x: 0, y: 0}, color: Stone::Black}, root.clone()).unwrap();
    // let child1: Ref<Rc<Node>> =  root.get_nth(0).unwrap();
    assert_eq!(format!("{:?}", child1), format!("{:?}", root.borrow().get_nth(0).unwrap()));
    {
        let mut marker: RefMut<Rc<Node>> = child1.borrow_mut();
        marker.mark = Some(BoardMarker { point: Point { x: 14, y: 14}, color: Stone::Black});
    }
    println!("{:?}", root.borrow().get_nth(0));
    //println!("{:?}", root.get_parent());
}
