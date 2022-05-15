use std::fmt::{Debug, Display};

use bevy::math::Vec2;

#[derive(Debug)]
pub struct QuadTree<T> {
    capacity: usize,
    root: QuadNode<T>,
}

#[derive(Debug)]
struct QuadNode<T> {
    objects: Vec<QuadObject<T>>,
    rect: QuadRect,

    subdivision: Option<Subdivision<T>>,
    level: usize,
}

#[derive(Debug)]
struct Subdivision<T> {
    top_left: Box<QuadNode<T>>,
    top_right: Box<QuadNode<T>>,
    bottom_left: Box<QuadNode<T>>,
    bottom_right: Box<QuadNode<T>>,
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
                objects: Vec::new(),
                rect: QuadRect { dim, pos },
                subdivision: None,
                level: 0,
            },
        }
    }

    pub fn add(&mut self, object: T, pos: Vec2, dim: Vec2) {
        let rect = QuadRect { pos, dim };
        let obj = QuadObject { object, rect };

        self.root.add(obj, self.capacity);
    }

    pub fn remove(&mut self, object: T) {
        todo!();
    }
}

impl<T> QuadNode<T>
where
    T: Clone,
{
    fn add(&mut self, object: QuadObject<T>, capacity: usize) {
        match &mut self.subdivision {
            None => {
                if self.objects.len() < capacity {
                    self.objects.push(object);
                } else {
                    self.subdivide(capacity);
                    self.add(object, capacity);
                }
            }

            Some(subdiv) => {
                if subdiv.top_left.rect.contains(&object.rect) {
                    subdiv.top_left.add(object, capacity);
                } else if subdiv.top_right.rect.contains(&object.rect) {
                    subdiv.top_right.add(object, capacity);
                } else if subdiv.bottom_left.rect.contains(&object.rect) {
                    subdiv.bottom_left.add(object, capacity);
                } else if subdiv.bottom_right.rect.contains(&object.rect) {
                    subdiv.bottom_right.add(object, capacity);
                } else {
                    self.objects.push(object);
                }
            }
        }
    }

    fn subdivide(&mut self, capacity: usize) {
        match self.subdivision {
            None => {
                let subdiv = Subdivision {
                    top_left: Box::new(QuadNode {
                        objects: Vec::new(),
                        rect: QuadRect {
                            pos: self.rect.pos,
                            dim: self.rect.dim / 2.0,
                        },
                        subdivision: None,
                        level: self.level + 1,
                    }),

                    top_right: Box::new(QuadNode {
                        objects: Vec::new(),
                        rect: QuadRect {
                            pos: Vec2::new(
                                self.rect.pos.x + self.rect.dim.x / 2.0,
                                self.rect.pos.y,
                            ),
                            dim: self.rect.dim / 2.0,
                        },
                        subdivision: None,
                        level: self.level + 1,
                    }),

                    bottom_left: Box::new(QuadNode {
                        objects: Vec::new(),
                        rect: QuadRect {
                            pos: Vec2::new(
                                self.rect.pos.x,
                                self.rect.pos.y + self.rect.dim.y / 2.0,
                            ),
                            dim: self.rect.dim / 2.0,
                        },
                        subdivision: None,
                        level: self.level + 1,
                    }),

                    bottom_right: Box::new(QuadNode {
                        objects: Vec::new(),
                        rect: QuadRect {
                            pos: self.rect.pos + self.rect.dim / 2.0,
                            dim: self.rect.dim / 2.0,
                        },
                        subdivision: None,
                        level: self.level + 1,
                    }),
                };

                self.subdivision = Some(subdiv);

                let v = self.objects.to_vec();

                self.objects.clear();

                for obj in v.into_iter() {
                    self.add(obj, capacity);
                }
            }

            Some(_) => panic!("Node should have been a Leaf"),
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
        for obj in self.objects.iter() {
            s.push_str(format!("{:?}; ", obj.object).as_str());
        }
        s.push_str("\n");

        s.push_str(format!("{}\trect: {:?}\n", tabs, self.rect).as_str());

        if let Some(subdiv) = &self.subdivision {
            s.push_str(format!("\n{}\tTopLeft    : {}\n", tabs, subdiv.top_left).as_str());
            s.push_str(format!("\n{}\tTopRight   : {}\n", tabs, subdiv.top_right).as_str());
            s.push_str(format!("\n{}\tBottomLeft : {}\n", tabs, subdiv.bottom_left).as_str());
            s.push_str(format!("\n{}\tBottomRight: {}\n", tabs, subdiv.bottom_right).as_str());
        }

        s.push_str(format!("{}}}\n", tabs).as_str());

        write!(f, "{}", s)
    }
}

impl QuadRect {
    fn contains(&self, other: &Self) -> bool {
        self.pos.x < other.pos.x
            && self.pos.x + self.dim.x > other.pos.x + other.dim.x
            && self.pos.y < other.pos.y
            && self.pos.y + self.dim.y > other.pos.y + other.dim.y
    }
}
