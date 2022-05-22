use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};

use bevy::math::Vec2;

#[derive(Debug)]
pub struct QuadTree<T> {
    capacity: usize,
    root: QuadNode,
    next_id: usize,

    objects: BTreeMap<usize, T>,
}

#[derive(Debug)]
struct QuadNode {
    object_ids: Vec<(usize, QuadRect)>,
    size: usize,
    rect: QuadRect,

    childs: Option<[Box<QuadNode>; 4]>,
    level: usize,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct QuadRect {
    pub pos: Vec2,
    pub dim: Vec2,
}

impl<T> QuadTree<T>
where
    T: Clone,
{
    pub fn new(capacity: usize, rect: QuadRect) -> Self {
        QuadTree {
            capacity,
            root: QuadNode {
                object_ids: Vec::new(),
                size: 0,
                rect,
                childs: None,
                level: 0,
            },
            next_id: 0,

            objects: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, object: T, rect: QuadRect) -> usize {
        let id = self.next_id;
        self.root.add(id, rect, self.capacity);

        if let Some(_) = self.objects.insert(id, object) {
            panic!("No unique ID");
        }

        self.next_id += 1;
        id
    }

    pub fn remove(&mut self, id: usize) -> Option<T> {
        let res = self.objects.remove(&id);
        if let Some(_) = res {
            self.root.remove(id, self.capacity);
        }

        res
    }

    pub fn remove_with_rect_contained(&mut self, id: usize, rect: QuadRect) -> Option<T> {
        let ret = self.objects.remove(&id);
        if let Some(_) = ret {
            self.root
                .remove_with_rect_contained(id, rect, self.capacity);
        }

        ret
    }

    pub fn remove_with_rect_overlaped(&mut self, id: usize, rect: QuadRect) -> Option<T> {
        let ret = self.objects.remove(&id);
        if let Some(_) = ret {
            self.root
                .remove_with_rect_overlaped(id, rect, self.capacity);
        }

        ret
    }

    pub fn search_overlaped(&self, rect: QuadRect) -> Vec<&T> {
        let vec = self.root.search_overlaped(rect);
        let mut ret = Vec::new();

        for id in vec {
            match self.objects.get(&id) {
                Some(obj) => ret.push(obj),
                None => panic!("Found invalid id"),
            }
        }

        ret
    }

    pub fn relocate(&mut self, id: usize, new_rect: QuadRect) {
        if self.root.remove(id, self.capacity) {
            self.root.add(id, new_rect, self.capacity);
        }
    }

    pub fn relocate_contained(&mut self, id: usize, current_rect: QuadRect, new_rect: QuadRect) {
        if self
            .root
            .remove_with_rect_contained(id, current_rect, self.capacity)
        {
            self.root.add(id, new_rect, self.capacity);
        }
    }

    pub fn relocate_overlaped(&mut self, id: usize, current_rect: QuadRect, new_rect: QuadRect) {
        if self
            .root
            .remove_with_rect_overlaped(id, current_rect, self.capacity)
        {
            self.root.add(id, new_rect, self.capacity);
        }
    }

    pub fn get_vertices(&self) -> Vec<[f32; 3]> {
        let pos = self.root.rect.pos;
        let dim = self.root.rect.dim;

        let mut ret = vec![
            // Top
            [pos.x, pos.y, 0.0],
            [pos.x + dim.x, pos.y, 0.0],
            // Down
            [pos.x, pos.y + dim.y, 0.0],
            [pos.x + dim.x, pos.y + dim.y, 0.0],
            // Left
            [pos.x, pos.y, 0.0],
            [pos.x, pos.y + dim.y, 0.0],
            // Right
            [pos.x + dim.x, pos.y, 0.0],
            [pos.x + dim.x, pos.y + dim.y, 0.0],
        ];

        if let Some(childs) = self.root.childs.borrow() {
            ret.append(&mut vec![
                // Vertical
                [
                    self.root.rect.pos.x + self.root.rect.dim.x / 2.0,
                    self.root.rect.pos.y,
                    0.0,
                ],
                [
                    self.root.rect.pos.x + self.root.rect.dim.x / 2.0,
                    self.root.rect.pos.y + self.root.rect.dim.y,
                    0.0,
                ],
                // Horizontal
                [
                    self.root.rect.pos.x,
                    self.root.rect.pos.y + self.root.rect.dim.y / 2.0,
                    0.0,
                ],
                [
                    self.root.rect.pos.x + self.root.rect.dim.x,
                    self.root.rect.pos.y + self.root.rect.dim.y / 2.0,
                    0.0,
                ],
            ]);

            for child in childs {
                ret.append(&mut child.get_vertices());
            }
        }

        ret
    }
}

impl QuadNode {
    fn add(&mut self, id: usize, rect: QuadRect, capacity: usize) {
        match &mut self.childs {
            None => {
                if self.object_ids.len() < capacity {
                    self.object_ids.push((id, rect));
                } else {
                    self.subdivide(capacity);
                    self.add(id, rect, capacity);
                }
            }

            Some(childs) => {
                for child in childs {
                    if child.rect.contains(&rect) {
                        child.add(id, rect, capacity);
                        return;
                    }
                }

                self.object_ids.push((id, rect));
            }
        }

        self.size += 1;
    }

    fn remove(&mut self, id: usize, capacity: usize) -> bool {
        match self.object_ids.iter().position(|&(idv, _)| id == idv) {
            Some(idx) => {
                self.size -= 1;
                self.object_ids.swap_remove(idx);
                self.repair(capacity);

                true
            }

            None => {
                let childs = self.childs.as_mut();

                match childs {
                    Some(childs) => {
                        for child in childs {
                            if child.remove(id, capacity) {
                                self.size -= 1;
                                self.repair(capacity);
                                return true;
                            }
                        }
                        false
                    }

                    None => false,
                }
            }
        }
    }

    fn remove_with_rect_contained(&mut self, id: usize, rect: QuadRect, capacity: usize) -> bool {
        let childs = self.childs.as_mut();

        if let Some(childs) = childs {
            for child in childs {
                if child.remove_with_rect_contained(id, rect, capacity) {
                    self.size -= 1;
                    self.repair(capacity);
                    return true;
                }
            }
        }

        if let Some(idx) = self.object_ids.iter().position(|&(id_vec, _)| id == id_vec) {
            let (_, rect_vec) = self.object_ids[idx];

            if rect.contains(&rect_vec) {
                self.object_ids.swap_remove(idx);
                self.size -= 1;
                self.repair(capacity);

                return true;
            }
        }

        false
    }

    fn remove_with_rect_overlaped(&mut self, id: usize, rect: QuadRect, capacity: usize) -> bool {
        let childs = self.childs.as_mut();

        if let Some(childs) = childs {
            for child in childs {
                if child.remove_with_rect_contained(id, rect, capacity) {
                    self.size -= 1;
                    self.repair(capacity);
                    return true;
                }
            }
        }

        if let Some(idx) = self.object_ids.iter().position(|&(id_vec, _)| id == id_vec) {
            let (_, rect_vec) = self.object_ids[idx];

            if rect.overlaps(&rect_vec) {
                self.object_ids.swap_remove(idx);
                self.size -= 1;
                self.repair(capacity);

                return true;
            }
        }

        false
    }

    fn search_overlaped(&self, rect: QuadRect) -> Vec<usize> {
        let mut ret = Vec::new();

        for &(id, obj_rect) in self.object_ids.iter() {
            if rect.overlaps(&obj_rect) {
                ret.push(id);
            }
        }

        if let Some(childs) = self.childs.borrow() {
            for child in childs {
                ret.append(&mut self.search_overlaped(rect));
            }
        }

        ret
    }

    fn subdivide(&mut self, capacity: usize) {
        match self.childs {
            None => {
                let childs = [
                    Box::new(QuadNode {
                        object_ids: Vec::new(),
                        size: 0,
                        rect: QuadRect {
                            pos: self.rect.pos,
                            dim: self.rect.dim / 2.0,
                        },
                        childs: None,
                        level: self.level + 1,
                    }),
                    Box::new(QuadNode {
                        object_ids: Vec::new(),
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
                        object_ids: Vec::new(),
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
                        object_ids: Vec::new(),
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

                let vec = self.object_ids.clone();

                self.object_ids.clear();

                for (id, rect) in vec.into_iter() {
                    self.add(id, rect, capacity);
                }
            }

            Some(_) => panic!("Node should have been a Leaf"),
        }
    }

    fn repair(&mut self, capacity: usize) {
        if self.size <= capacity {
            self.object_ids = self.pull_all_up();
            self.childs = None;
        }
    }

    fn pull_all_up(&mut self) -> Vec<(usize, QuadRect)> {
        let mut res = Vec::new();

        if let Some(childs) = self.childs.as_mut() {
            for child in childs {
                res.append(&mut child.pull_all_up());
            }
        }
        res.append(&mut self.object_ids);

        res
    }

    fn get_vertices(&self) -> Vec<[f32; 3]> {
        let mut ret = Vec::new();

        if let Some(childs) = self.childs.borrow() {
            ret.append(&mut vec![
                // Vertical
                [
                    self.rect.pos.x + self.rect.dim.x / 2.0,
                    self.rect.pos.y,
                    0.0,
                ],
                [
                    self.rect.pos.x + self.rect.dim.x / 2.0,
                    self.rect.pos.y + self.rect.dim.y,
                    0.0,
                ],
                // Horizontal
                [
                    self.rect.pos.x,
                    self.rect.pos.y + self.rect.dim.y / 2.0,
                    0.0,
                ],
                [
                    self.rect.pos.x + self.rect.dim.x,
                    self.rect.pos.y + self.rect.dim.y / 2.0,
                    0.0,
                ],
            ]);

            for child in childs {
                ret.append(&mut child.get_vertices());
            }
        }

        ret
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

        s.push_str(format!("\tobjects: {:?}\n", self.objects).as_str());

        s.push_str(format!("\troot: {}", self.root).as_str());

        s.push_str("}\n");

        write!(f, "{}", s)
    }
}

impl Display for QuadNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = std::string::String::new();

        let mut tabs = "\t".to_string();
        for _ in 0..self.level {
            tabs.push_str("\t");
        }

        s.push_str("{\n");

        s.push_str(format!("{}\tids: ", tabs).as_str());
        for (id, _) in self.object_ids.iter() {
            s.push_str(format!("{}; ", id).as_str());
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

    fn overlaps(&self, other: &Self) -> bool {
        self.pos.x <= other.pos.x + other.dim.x
            && self.pos.x + self.dim.x >= other.pos.x
            && self.pos.y <= other.pos.y + other.dim.y
            && self.pos.y + self.dim.y >= other.pos.y
    }
}
