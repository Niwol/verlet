use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};

use bevy::math::Vec2;

#[derive(Debug)]
pub struct QuadTree<T> {
    capacity: usize,
    root: QuadNode<T>,
    next_id: usize,
}

#[derive(Debug)]
struct QuadNode<T> {
    objects: BTreeMap<usize, QuadObject<T>>,
    size: usize,
    rect: QuadRect,

    childs: Option<[Box<QuadNode<T>>; 4]>,
    level: usize,
}

#[derive(Copy, Clone, Debug)]
struct QuadRect {
    pos: Vec2,
    dim: Vec2,
}

#[derive(Debug, Clone)]
struct QuadObject<T> {
    object: T,
    rect: QuadRect,
}

impl<T> QuadTree<T>
where
    T: Clone,
{
    pub fn new(capacity: usize, pos: Vec2, dim: Vec2) -> Self {
        QuadTree {
            capacity,
            root: QuadNode {
                objects: BTreeMap::new(),
                size: 0,
                rect: QuadRect { dim, pos },
                childs: None,
                level: 0,
            },
            next_id: 0,
        }
    }

    pub fn add(&mut self, object: T, pos: Vec2, dim: Vec2) -> usize {
        let rect = QuadRect { pos, dim };
        let obj = QuadObject { object, rect };

        let id = self.next_id;
        self.root.add(obj, id, self.capacity);

        self.next_id += 1;
        id
    }

    pub fn remove(&mut self, id: usize) -> Option<T> {
        self.root.remove(id, self.capacity)
    }

    pub fn remove_with_rect_contained(&mut self, id: usize, pos: Vec2, dim: Vec2) -> Option<T> {
        let rect = QuadRect { pos, dim };
        self.root
            .remove_with_rect_contained(id, rect, self.capacity)
    }
}

impl<T> QuadNode<T>
where
    T: Clone,
{
    fn add(&mut self, object: QuadObject<T>, id: usize, capacity: usize) {
        match &mut self.childs {
            None => {
                if self.objects.len() < capacity {
                    self.objects.insert(id, object);
                } else {
                    self.subdivide(capacity);
                    self.add(object, id, capacity);
                }
            }

            Some(childs) => {
                for child in childs {
                    if child.rect.contains(&object.rect) {
                        child.add(object, id, capacity);
                        return;
                    }
                }

                self.objects.insert(id, object);
            }
        }

        self.size += 1;
    }

    fn remove_all(&mut self) -> BTreeMap<usize, QuadObject<T>> {
        let mut res = BTreeMap::new();

        if let Some(childs) = self.childs.as_mut() {
            for child in childs {
                res.append(&mut child.remove_all());
            }
        }

        res.append(&mut self.objects);

        res
    }

    fn remove(&mut self, id: usize, capacity: usize) -> Option<T> {
        match self.objects.remove(&id) {
            Some(obj) => {
                self.size -= 1;
                self.repair(capacity);
                Some(obj.object)
            }

            None => {
                let childs = self.childs.as_mut();

                match childs {
                    Some(childs) => {
                        for child in childs {
                            if let Some(obj) = child.remove(id, capacity) {
                                self.size -= 1;
                                self.repair(capacity);
                                return Some(obj);
                            }
                        }
                        None
                    }

                    None => None,
                }
            }
        }
    }

    fn remove_with_rect_contained(
        &mut self,
        id: usize,
        rect: QuadRect,
        capacity: usize,
    ) -> Option<T> {
        match self.objects.get(&id) {
            Some(obj) => {
                if rect.contains(&obj.rect) {
                    self.size -= 1;
                    self.repair(capacity);

                    Some(self.objects.remove(&id).unwrap().object)
                } else {
                    None
                }
            }

            None => {
                let childs = self.childs.as_mut();

                match childs {
                    Some(childs) => {
                        for child in childs {
                            if child.rect.contains(&rect) {
                                let res = child.remove_with_rect_contained(id, rect, capacity);

                                self.size -= 1;
                                self.repair(capacity);

                                return res;
                            }
                        }

                        None
                    }

                    None => None,
                }
            }
        }
    }

    fn subdivide(&mut self, capacity: usize) {
        match self.childs {
            None => {
                let childs = [
                    Box::new(QuadNode {
                        objects: BTreeMap::new(),
                        size: 0,
                        rect: QuadRect {
                            pos: self.rect.pos,
                            dim: self.rect.dim / 2.0,
                        },
                        childs: None,
                        level: self.level + 1,
                    }),
                    Box::new(QuadNode {
                        objects: BTreeMap::new(),
                        size: 0,
                        rect: QuadRect {
                            pos: Vec2::new(
                                self.rect.pos.x + self.rect.dim.x / 2.0,
                                self.rect.pos.y,
                            ),
                            dim: self.rect.dim / 2.0,
                        },
                        childs: None,
                        level: self.level + 1,
                    }),
                    Box::new(QuadNode {
                        objects: BTreeMap::new(),
                        size: 0,
                        rect: QuadRect {
                            pos: Vec2::new(
                                self.rect.pos.x,
                                self.rect.pos.y + self.rect.dim.y / 2.0,
                            ),
                            dim: self.rect.dim / 2.0,
                        },
                        childs: None,
                        level: self.level + 1,
                    }),
                    Box::new(QuadNode {
                        objects: BTreeMap::new(),
                        size: 0,
                        rect: QuadRect {
                            pos: self.rect.pos + self.rect.dim / 2.0,
                            dim: self.rect.dim / 2.0,
                        },
                        childs: None,
                        level: self.level + 1,
                    }),
                ];

                self.childs = Some(childs);

                let map = self.objects.clone();

                self.objects.clear();

                for (id, obj) in map.into_iter() {
                    self.add(obj, id, capacity);
                }
            }

            Some(_) => panic!("Node should have been a Leaf"),
        }
    }

    fn repair(&mut self, capacity: usize) {
        if self.size <= capacity {
            if let Some(_) = self.childs {
                let vec = self.remove_all();

                self.childs = None;
                for (id, obj) in vec {
                    self.add(obj, id, capacity);
                }
            }
        }
    }
}

impl<T> Display for QuadTree<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = std::string::String::new();

        s.push_str("QuadTree {\n");

        s.push_str(format!("\tcapacity: {}\n", self.capacity).as_str());
        s.push_str(format!("\troot: {}", self.root).as_str());

        s.push_str("}\n");

        write!(f, "{}", s)
    }
}

impl<T> Display for QuadNode<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = std::string::String::new();

        let mut tabs = "\t".to_string();
        for _ in 0..self.level {
            tabs.push_str("\t");
        }

        s.push_str("{\n");

        s.push_str(format!("{}\tobjects: ", tabs).as_str());
        for (_, obj) in self.objects.iter() {
            s.push_str(format!("{:?}; ", obj.object).as_str());
        }
        s.push_str("\n");

        s.push_str(format!("{}\trect: {:?}\n", tabs, self.rect).as_str());

        if let Some(childs) = &self.childs {
            s.push_str(format!("\n{}\tTopLeft    : {}\n", tabs, childs[0]).as_str());
            s.push_str(format!("\n{}\tTopRight   : {}\n", tabs, childs[1]).as_str());
            s.push_str(format!("\n{}\tBottomLeft : {}\n", tabs, childs[2]).as_str());
            s.push_str(format!("\n{}\tBottomRight: {}\n", tabs, childs[3]).as_str());
        }

        s.push_str(format!("{}}}\n", tabs).as_str());

        write!(f, "{}", s)
    }
}

impl QuadRect {
    fn contains(&self, other: &Self) -> bool {
        self.pos.x <= other.pos.x
            && self.pos.x + self.dim.x >= other.pos.x + other.dim.x
            && self.pos.y <= other.pos.y
            && self.pos.y + self.dim.y >= other.pos.y + other.dim.y
    }
}
