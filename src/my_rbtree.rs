use std::{fmt::Display, ptr::NonNull, rc::Weak};

#[derive(Clone, Copy)]
enum RbColor {
    Red,
    Black,
}

impl PartialEq for RbColor {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (RbColor::Red, RbColor::Red) | (RbColor::Black, RbColor::Black)
        )
    }
}

struct RbNode<T: Ord> {
    value: T,
    color: RbColor,
    left: Option<NonNull<RbNode<T>>>,
    right: Option<NonNull<RbNode<T>>>,
    parent: Option<NonNull<RbNode<T>>>,
}

impl<T: Ord> RbNode<T> {
    fn new(value: T) -> Self {
        RbNode {
            value,
            color: RbColor::Red,
            left: None,
            right: None,
            parent: None,
        }
    }
}

pub struct RbTree<T: Ord> {
    root: Option<NonNull<RbNode<T>>>,
    size: usize,
}

impl<T: Ord> RbTree<T> {
    pub fn new() -> Self {
        RbTree {
            root: None,
            size: 0,
        }
    }

    // 向上修复成2-3-4树
    fn insert_fixup(
        cur_node: NonNull<RbNode<T>>,
        parent_ref: &mut Option<NonNull<RbNode<T>>>,
        uncle_ref: &mut Option<NonNull<RbNode<T>>>,
        gp_ref: &mut Option<NonNull<RbNode<T>>>,
    ) -> Option<NonNull<RbNode<T>>> {
        let parent_ptr = (parent_ref.unwrap()).as_ptr();
        let parent_value = unsafe { &(*parent_ptr).value };
        let parent_color = unsafe { (*parent_ptr).color };
        let current_color = unsafe { (*cur_node.as_ptr()).color };
        let current_value = unsafe { &(*cur_node.as_ptr()).value };
        let parent_left = unsafe { (*parent_ptr).left };
        let parent_right = unsafe { (*parent_ptr).right };
        // 不需要继续向上合并的两种情况
        // 1. 插入后是3-节点
        // 2. 插入后是4-节点

        if current_color == RbColor::Red && parent_color == RbColor::Red {
            // 连续红节点
            // 1. 可能是需要重新排序的4-节点
            // 2. 可能是需要向上合并的5-节点
            // 取决于叔叔节点的颜色
            let gp = unsafe { (*parent_ptr).parent }.expect("连续红节点到达根节点");
            let gp_ptr = gp.as_ptr();
            // 由于根节点始终是黑色，所以不可能出现连续红节点到达根节点的情况

            let gp_value = unsafe { &(*gp.as_ptr()).value };

            let (uncle_ptr, uncle_color) = match uncle_ref {
                Some(uncle) => {
                    let ptr = uncle.as_ptr();
                    (ptr, unsafe { (*ptr).color })
                }
                None => (std::ptr::null_mut(), RbColor::Black),
            };

            if uncle_color == RbColor::Red {
                // 是5-节点，向上合并
                // 两种情况的操作是一样的

                // 第一种：
                // 2-3-4树
                //      .         [. G]
                //      |           / \
                // [C P G U] => [C P] [U]
                //    |            |
                //    B            B
                // 红黑树
                //     G.B          G.R
                //    / \          / \
                //   P.R U.R =>  P.B  U.B
                //  / \         /   \
                // C.R B       C.R   B

                // 第二种：
                // 2-3-4树
                //    .         [. G]
                //    |           / \
                // [U G P C] => [U] [P C]
                //      |            |
                //      B            B
                // 红黑树
                //     G.B          G.R
                //    / \          / \
                //   U.R P.R =>  U.B  P.B
                //      / \          / \
                //     B   C.R      B   C.R
                unsafe {
                    (*parent_ptr).color = RbColor::Black;
                    (*uncle_ptr).color = RbColor::Black;
                    (*gp_ptr).color = RbColor::Red;
                }

                // 剩下的交给递归
                return Some(gp);
            } else {
                // 是4-节点，重新排序
                if current_value < parent_value {
                    // 左倾
                    if parent_value < gp_value {
                        // 2-3-4树
                        //       .          .
                        //       |          |
                        //  [C P G]  =>  [C P G]
                        //     |  \          / \
                        //     B   U        B   U

                        // 红黑树
                        //     G.B        P.B
                        //    / \        / \
                        //   P.R U.B => C.R G.R
                        //  / \            / \
                        // C.R B          B   U.B
                        // 右旋
                        unsafe {
                            (*parent_ptr).color = RbColor::Black;
                            (*gp_ptr).color = RbColor::Red;

                            *gp_ref = (*gp_ptr).left;

                            (*parent_ptr).parent = (*gp_ptr).parent;
                            (*gp_ptr).parent = *parent_ref;

                            (*gp_ptr).left = parent_right;
                            (*parent_ptr).right = Some(gp);
                        }
                    } else {
                        // 2-3-4树
                        //   .              .
                        //   |              |
                        //  [G C P]  =>  [G C P]
                        //  /     \      /     \
                        // U       B    U       B

                        // 红黑树
                        //     G.B        C.B
                        //    / \        / \
                        //   U.B P.R => G.R P.R
                        //      / \    /     \
                        //     C.R B  U.B     B
                        // 右旋+左旋
                        unsafe {
                            let cur_ptr = cur_node.as_ptr();
                            (*cur_ptr).color = RbColor::Black;
                            (*gp_ptr).color = RbColor::Red;

                            (*parent_ptr).left = (*cur_ptr).right;
                            (*gp_ptr).right = (*cur_ptr).left;

                            (*cur_ptr).left = Some(gp);
                            (*cur_ptr).right = (*cur_ptr).parent;

                            (*cur_ptr).parent = (*gp_ptr).parent;
                            (*parent_ptr).parent = Some(cur_node);
                            (*gp_ptr).parent = Some(cur_node);

                            *gp_ref = Some(cur_node);
                        }
                    }
                } else {
                    if parent_value < gp_value {
                        // 2-3-4树
                        //       .          .
                        //       |          |
                        //  [P C G]  =>  [P C G]
                        //  /     \      /     \
                        // B       U    B       U

                        // 红黑树
                        //     G.B        C.B
                        //    / \        / \
                        //   P.R U.B => P.R G.R
                        //  / \        /     \
                        // B   C.R    B       U.B
                        // 左旋+右旋
                        unsafe {
                            let cur_ptr = cur_node.as_ptr();
                            (*cur_ptr).color = RbColor::Black;
                            (*gp_ptr).color = RbColor::Red;

                            (*parent_ptr).right = (*cur_ptr).left;
                            (*gp_ptr).left = (*cur_ptr).right;

                            (*cur_ptr).left = (*cur_ptr).parent;
                            (*cur_ptr).right = (*parent_ptr).parent;

                            (*cur_ptr).parent = (*gp_ptr).parent;

                            (*parent_ptr).parent = Some(cur_node);
                            (*gp_ptr).parent = Some(cur_node);

                            (*gp_ref) = Some(cur_node);
                        }
                    } else {
                        // 2-3-4树
                        //   .              .
                        //   |              |
                        //  [G P C]  =>  [G P C]
                        //  /  |         /  |
                        // U   B        U   B

                        // 红黑树
                        //     G.B         P.B
                        //    / \         / \
                        //   U.B P.R =>  G.R C.R
                        //      / \     / \
                        //     B   C.R U.B B
                        // 左旋
                        unsafe {
                            (*parent_ptr).color = RbColor::Black;
                            (*gp_ptr).color = RbColor::Red;

                            (*parent_ptr).parent = (*gp_ptr).parent;
                            (*gp_ptr).parent = *parent_ref;

                            (*gp_ref) = *parent_ref;

                            (*gp_ptr).right = parent_left;
                            (*parent_ptr).left = Some(gp);
                        }
                    }
                }
            }
        }
        None
    }

    // 先按二叉树的方式插入，不管平衡
    fn insert_new(
        new_node: NonNull<RbNode<T>>,
        parent_ref: &mut Option<NonNull<RbNode<T>>>,
        uncle_ref: &mut Option<NonNull<RbNode<T>>>,
        grand_parent_ref: &mut Option<NonNull<RbNode<T>>>,
    ) -> Option<NonNull<RbNode<T>>> {
        let new_value = unsafe { &(*new_node.as_ptr()).value };
        let parent_ptr = (parent_ref.unwrap()).as_ptr();
        let parent_value = unsafe { &(*parent_ptr).value };
        let parent_left = unsafe { &mut (*parent_ptr).left };
        let parent_right = unsafe { &mut (*parent_ptr).right };

        let check = if new_value < parent_value {
            if parent_left.is_some() {
                Self::insert_new(new_node, parent_left, parent_right, parent_ref)
            } else {
                // 左为空，直接插入
                unsafe {
                    (*parent_ptr).left = Some(new_node);
                    (*new_node.as_ptr()).parent = Some(NonNull::new_unchecked(parent_ptr));
                }
                Self::insert_fixup(new_node, parent_ref, uncle_ref, grand_parent_ref)
            }
        } else {
            if parent_right.is_some() {
                Self::insert_new(new_node, parent_right, parent_left, parent_ref)
            } else {
                // 右为空，直接插入
                unsafe {
                    (*parent_ptr).right = Some(new_node);
                    (*new_node.as_ptr()).parent = Some(NonNull::new_unchecked(parent_ptr));
                }
                Self::insert_fixup(new_node, parent_ref, uncle_ref, grand_parent_ref)
            }
        };
        if let Some(check_node) = check {
            let _parent_ptr = match unsafe { (*check_node.as_ptr()).parent } {
                Some(ptr) => ptr.as_ptr(),
                None => {
                    return None;
                }
            };
            if _parent_ptr == parent_ptr {
                unsafe {
                    if let Some(left) = (*parent_ptr).left {
                        if left == check_node {
                            return Self::insert_fixup(
                                left,
                                parent_ref,
                                uncle_ref,
                                grand_parent_ref,
                            );
                        }
                    }
                    if let Some(right) = (*parent_ptr).right {
                        if right == check_node {
                            return Self::insert_fixup(
                                right,
                                parent_ref,
                                uncle_ref,
                                grand_parent_ref,
                            );
                        }
                    }
                }
            } else {
                return check;
            }
        }
        None
    }

    pub fn insert(&mut self, key: T) {
        let new_node = RbNode::new(key);
        let new_node = Box::new(new_node);
        let new_node_raw = Box::into_raw(new_node);
        let new_node = unsafe { NonNull::new_unchecked(new_node_raw) };

        match self.root {
            Some(_) => {
                Self::insert_new(new_node, &mut self.root, &mut None, &mut None);
            }
            None => {
                self.root = Some(new_node);
            }
        }
        unsafe {
            (*self.root.unwrap_unchecked().as_ptr()).color = RbColor::Black;
        }
        self.size += 1;
    }

    fn change_child(
        &mut self,
        parent: Option<NonNull<RbNode<T>>>,
        old_child: NonNull<RbNode<T>>,
        new_child: Option<NonNull<RbNode<T>>>,
    ) {
        if let Some(parent) = parent {
            let parent_ptr = parent.as_ptr();
            unsafe {
                if let Some(right) = (*parent_ptr).right
                    && right == old_child
                {
                    (*parent_ptr).right = new_child;
                } else if let Some(left) = (*parent_ptr).left
                    && left == old_child
                {
                    (*parent_ptr).left = new_child;
                } else {
                    panic!("change_child: old_child is not a child of parent");
                }
                if let Some(new_child_node) = new_child {
                    let new_child_ptr = new_child_node.as_ptr();
                    (*new_child_ptr).parent = Some(parent);
                }
            }
        } else {
            // 根节点
            self.root = new_child;
            if let Some(new_child_node) = new_child {
                let new_child_ptr = new_child_node.as_ptr();
                unsafe {
                    (*new_child_ptr).parent = None;
                }
            }
        }
    }

    // 从右子树中找到最左节点
    fn find_left_node_right(&self, right: NonNull<RbNode<T>>) -> NonNull<RbNode<T>> {
        let mut current = right;
        unsafe {
            while let Some(left) = (*current.as_ptr()).left {
                current = left;
            }
        }
        current
    }

    fn delete_fixup(&mut self, parent: NonNull<RbNode<T>>) {
        let mut parent = parent;
        let mut sibling;
        let mut node = None;
        loop {
            let parent_ptr = parent.as_ptr();
            sibling = unsafe { (*parent_ptr).right };

            unsafe {
                let parent_ptr = parent.as_ptr();

                let flag = match (sibling, node) {
                    (Some(_), None) => {true},
                    (None, Some(_)) => {false},
                    (Some(_sibling), Some(_node)) => {_sibling != _node},
                    (None, None) => {false},
                };

                if flag {
                    let sibling_ptr = sibling.unwrap_unchecked().as_ptr();

                    if let RbColor::Red = (*sibling_ptr).color {
                        // 兄弟节点为红色，通过旋转转换到黑色兄弟节点的情况

                        // 2-3-4树
                        //     .             .
                        //     |             |
                        //    [P S]  =>   [P S]
                        //    /  |  \     / | \
                        //   N   A   B   N  A  B

                        // 红黑树
                        //     P.B          S.B
                        //    / \          / \
                        //   N.B S.R =>  P.R  B.B
                        //      / \      / \
                        //     A.B B.B  N.B A.B
                        self.change_child((*parent_ptr).parent, parent, sibling);

                        (*parent_ptr).right = (*sibling_ptr).left;
                        (*sibling_ptr).left = Some(parent);
                        (*parent_ptr).parent = sibling;
                        (*sibling_ptr).color = RbColor::Black;
                        (*parent_ptr).color = RbColor::Red;
                        if let Some(right) = (*parent_ptr).right {
                            let right_ptr = right.as_ptr();
                            (*right_ptr).parent = Some(parent);
                        }

                        sibling = (*parent_ptr).right;
                    }

                    let sibling_ptr = sibling.unwrap().as_ptr();

                    let sl_color = if let Some(sl) = (*sibling_ptr).left {
                        let sl_ptr = sl.as_ptr();
                        (*sl_ptr).color
                    } else {
                        RbColor::Black
                    };
                    let sr_color = if let Some(sr) = (*sibling_ptr).right {
                        let sr_ptr = sr.as_ptr();
                        (*sr_ptr).color
                    } else {
                        RbColor::Black
                    };
                    match (sl_color, sr_color) {
                        (RbColor::Black, RbColor::Black) => {
                            // 情况一: 兄弟节点的两个子节点全黑
                            // 目标：右子树黑高减一（整体黑高减一）
                            // 兄弟节点存在且全黑
                            // 将兄弟节点染成红色(在2-3-4树中与父节点合并），
                            // 相当于将整个子树黑高减一，向上继续平衡

                            // 2-3-4树
                            //     .         .
                            //     |         |
                            //     P  =>    [P S]
                            //    / \       / | \
                            //   N   S     N  SL SR
                            //      / \
                            //     SL  SR
                            (*sibling_ptr).color = RbColor::Red;
                            match (*parent_ptr).color {
                                RbColor::Red => {
                                    (*parent_ptr).color = RbColor::Black;
                                },
                                RbColor::Black => {
                                    (node, parent) = match (*parent_ptr).parent {
                                        Some(_parent) => {(Some(parent), _parent)},
                                        None => {break;},
                                    };
                                    continue;
                                }
                            }
                            break;
                        }
                        (RbColor::Red, RbColor::Black) => {
                            let sl = (*sibling_ptr).left.unwrap_unchecked();
                            let sl_ptr = sl.as_ptr();
                            // 情况二: 兄弟节点的近子节点为红色，远子节点为黑色
                            // 目标：左子树黑高加一（整体黑高不变）
                            // 兄弟节点左子节点为红色，右子节点为空（黑色）
                            // 将左节点作为子树的新根

                            // 2-3-4树
                            //     .              .
                            //     |              |
                            //     P    =>        SL
                            //   /   \          /    \
                            //  N [SL S]       P      S
                            //    /  | \      / \    / \
                            //   A   B  SR   N   A  B   SR

                            // 红黑树
                            //     P.B          SL.B
                            //    / \          /    \
                            //   N.B S.B =>   P.B    S.B
                            //      / \      / \    / \
                            //    SL.R SR.B N.B A  B   SR.B
                            //   /  \
                            //  A    B
                            self.change_child((*parent_ptr).parent, parent, Some(sl));

                            (*sibling_ptr).left = (*sl_ptr).right;
                            (*parent_ptr).right = (*sl_ptr).left;
                            (*sl_ptr).right = sibling;
                            (*sl_ptr).left = Some(parent);

                            if let Some(right) = (*parent_ptr).right {
                                let right_ptr = right.as_ptr();
                                (*right_ptr).parent = Some(parent);
                            }
                            if let Some(left) = (*sibling_ptr).left {
                                let left_ptr = left.as_ptr();
                                (*left_ptr).parent = sibling;
                            }

                            (*parent_ptr).parent = Some(sl);
                            (*sibling_ptr).parent = Some(sl);
                            (*sl_ptr).color = RbColor::Black;
                            break;
                        }
                        (RbColor::Black, RbColor::Red) | (RbColor::Red, RbColor::Red) => {
                            let sr_ptr = (*sibling_ptr).right.unwrap_unchecked().as_ptr();
                            // 情况三: 兄弟节点的左子节点为黑色，右子节点为红色
                            // 目标：左子树黑高加一（整体黑高不变）
                            // 兄弟节点右子节点为红色，左子节点为黑色
                            // 将右节点作为子树的新根

                            // 情况四：兄弟节点的左右两个节点都是红色
                            // 目标：左子树黑高加一（整体黑高不变）
                            // 和情况三共用处理逻辑

                            // 2-3-4树
                            //     .              .
                            //     |              |
                            //     P    =>        S
                            //   /   \          /   \
                            //  N   [S SR]     P     SR
                            //      /  | \    / \    / \
                            //     SL  A  B  N   SL A   B

                            // 红黑树
                            //     P.B           S.B
                            //    / \          /     \
                            //   N.B S.B =>   P.B    SR.B
                            //      / \      / \     / \
                            //    SL.B SR.R N  SL.B A   B
                            //         / \
                            //        A   B
                            self.change_child((*parent_ptr).parent, parent, sibling);

                            (*parent_ptr).right = (*sibling_ptr).left;
                            (*parent_ptr).parent = sibling;
                            (*sibling_ptr).left = Some(parent);

                            if let Some(left) = (*parent_ptr).right {
                                let left_ptr = left.as_ptr();
                                (*left_ptr).parent = Some(parent);
                            }
                            if let Some(right) = (*sibling_ptr).right {
                                let right_ptr = right.as_ptr();
                                (*right_ptr).parent = sibling;
                            }
                            
                            (*sr_ptr).parent = sibling;
                            (*sr_ptr).color = RbColor::Black;
                            break;
                        }
                    };
                } else {
                    // 对称方向
                    sibling = (*parent_ptr).left;
                    let sibling_ptr = sibling.unwrap().as_ptr();

                    // 预处理，消除红色兄弟节点的情况
                    if let RbColor::Red = (*sibling_ptr).color {
                        // 兄弟节点为红色，通过旋转转换到黑色兄弟节点的情况

                        // 2-3-4树
                        //       .         .
                        //       |         |
                        //    [S P]  =>   [S P]
                        //    /  |  \     / | \
                        //   A   B   N   A  B  N

                        // 红黑树
                        //     P.B          S.B
                        //    / \          / \
                        //   S.R N    =>  A.B P.R
                        //  / \              / \
                        // A.B B.B          B.B N
                        self.change_child((*parent_ptr).parent, parent, sibling);

                        (*parent_ptr).left = (*sibling_ptr).right;
                        (*sibling_ptr).right = Some(parent);
                        (*parent_ptr).parent = sibling;
                        (*sibling_ptr).color = RbColor::Black;
                        (*parent_ptr).color = RbColor::Red;
                        if let Some(left) = (*parent_ptr).left {
                            let left_ptr = left.as_ptr();
                            (*left_ptr).parent = Some(parent);
                        }

                        sibling = (*parent_ptr).left;
                    }

                    let sibling_ptr = sibling.unwrap().as_ptr();

                    let sl_color = if let Some(sl) = (*sibling_ptr).left {
                        let sl_ptr = sl.as_ptr();
                        (*sl_ptr).color
                    } else {
                        RbColor::Black
                    };
                    let sr_color = if let Some(sr) = (*sibling_ptr).right {
                        let sr_ptr = sr.as_ptr();
                        (*sr_ptr).color
                    } else {
                        RbColor::Black
                    };
                    match (sl_color, sr_color) {
                        (RbColor::Black, RbColor::Black) => {
                            // 情况一: 兄弟节点的两个子节点全黑
                            // 目标：左子树黑高减一（整体黑高减一）
                            // 兄弟节点存在且全黑
                            // 将兄弟节点染成红色(在2-3-4树中与父节点合并），
                            // 相当于将整个子树黑高减一，向上继续平衡

                            // 2-3-4树
                            //     .           .
                            //     |           |
                            //     P  =>    [S P]
                            //    / \       / | \
                            //   S   N     SL SR N
                            //  / \
                            // SL  SR
                            (*sibling_ptr).color = RbColor::Red;
                            match (*parent_ptr).color {
                                RbColor::Red => {
                                    (*parent_ptr).color = RbColor::Black;
                                },
                                RbColor::Black => {
                                    (node, parent) = match (*parent_ptr).parent {
                                        Some(_parent) => {(Some(parent), _parent)},
                                        None => {break;},
                                    };
                                    continue;
                                }
                            }
                            break;
                        }
                        (RbColor::Black, RbColor::Red) => {
                            let sr = (*sibling_ptr).right.unwrap_unchecked();
                            let sr_ptr = sr.as_ptr();
                            // 情况二: 兄弟节点的远子节点为黑色，近子节点为红色
                            // 目标：右子树黑高加一（整体黑高不变）
                            // 兄弟节点左子节点为红色，右子节点为黑色
                            // 将右节点作为子树的新根

                            // 2-3-4树
                            //       .             .
                            //       |             |
                            //       P            SR
                            //    /     \  =>    /   \
                            //   [S SR]  N      S     P
                            //   /  | \        / \   / \
                            //  SL  A  B      SL  A B   N

                            // 红黑树
                            //      P.B           SR.B
                            //     / \          /     \
                            //    S.B N.B =>   S.B     P.B
                            //   / \          / \     / \
                            // SL.B SR.R    SL.B A   B   N
                            //      / \
                            //     A   B
                            self.change_child((*parent_ptr).parent, parent, Some(sr));

                            (*parent_ptr).left = (*sr_ptr).right;
                            (*sibling_ptr).right = (*sr_ptr).left;
                            (*sr_ptr).right = Some(parent);
                            (*sr_ptr).left = sibling;

                            if let Some(left) = (*parent_ptr).left {
                                let left_ptr = left.as_ptr();
                                (*left_ptr).parent = Some(parent);
                            }
                            if let Some(right) = (*sibling_ptr).right {
                                let right_ptr = right.as_ptr();
                                (*right_ptr).parent = sibling;
                            }
                            (*parent_ptr).parent = Some(sr);
                            (*sibling_ptr).parent = Some(sr);
                            (*sr_ptr).color = RbColor::Black;
                            break;
                        }
                        (RbColor::Red, RbColor::Black) | (RbColor::Red, RbColor::Red) => {
                            let sl = (*sibling_ptr).left.unwrap_unchecked();
                            let sl_ptr = sl.as_ptr();
                            // 情况三: 兄弟节点的近子节点为黑色，远子节点为红色
                            // 目标：右子树黑高加一（整体黑高不变）
                            // 兄弟节点左子节点为红色，右子节点为黑色
                            // 将兄弟节点作为整个子树的新根

                            // 情况四：兄弟节点的左右两个节点都是红色
                            // 目标：右子树黑高加一（整体黑高不变）
                            // 和情况三共用处理逻辑

                            // 2-3-4树
                            //       .              .
                            //       |              |
                            //       P    =>        S
                            //      / \           /   \
                            //  [SL S] N         SL    P
                            //  / |  \          / \   / \
                            // A  B   SR       A   B SR  N

                            // 红黑树
                            //        P.B             S.B
                            //       / \             /   \
                            //      S.B N.B   =>   SL.B   P.B
                            //     /  \           / \    /  \
                            //    SL.R SR.B      A  B   SR.B N
                            //   /  \
                            //  A    B
                            self.change_child((*parent_ptr).parent, parent, sibling);

                            (*parent_ptr).left = (*sibling_ptr).right;
                            (*parent_ptr).parent = sibling;
                            (*sibling_ptr).right = Some(parent);

                            if let Some(left) = (*parent_ptr).left {
                                let left_ptr = left.as_ptr();
                                (*left_ptr).parent = Some(parent);
                            }
                            if let Some(left) = (*sibling_ptr).left {
                                let left_ptr = left.as_ptr();
                                (*left_ptr).parent = sibling;
                            }

                            (*sl_ptr).parent = sibling;
                            (*sl_ptr).color = RbColor::Black;
                            break;
                        }
                    };
                }
            }
        }
    }

    fn delete_node(&mut self, node_ptr: *mut RbNode<T>) {
        let mut rebalance = None;
        let parent = unsafe { (*node_ptr).parent };

        let (left, right) = unsafe { ((*node_ptr).left, (*node_ptr).right) };

        match (left, right) {
            (Some(left_node), Some(right_node)) => {
                let left_ptr = left_node.as_ptr();
                let right_ptr = right_node.as_ptr();
                let child_ptr = right_ptr;

                // 选择中继节点
                let successor_ptr;
                let parent_ptr;
                let successor_right;
                let (child_left, child_right) = unsafe { ((*child_ptr).left, (*child_ptr).right) };

                successor_ptr = self.find_left_node_right(right_node).as_ptr();
                if successor_ptr == right_ptr {
                    parent_ptr = child_ptr;
                    successor_right = child_right;
                } else {
                    // 与后继节点交换
                    unsafe {
                        parent_ptr = (*successor_ptr).parent.unwrap().as_ptr();
                        successor_right = (*successor_ptr).right;
                        (*parent_ptr).left = successor_right;
                        (*successor_ptr).right = Some(right_node);
                        (*right_ptr).parent = Some(NonNull::new_unchecked(successor_ptr));
                    }
                }

                unsafe {
                    (*successor_ptr).left = Some(left_node);
                    (*left_ptr).parent = Some(NonNull::new_unchecked(successor_ptr));
                    self.change_child(
                        parent,
                        NonNull::new_unchecked(node_ptr),
                        Some(NonNull::new_unchecked(successor_ptr)),
                    );
                }

                match successor_right {
                    Some(sr) => {
                        // 如果successor是红节点，他的右节点必然是黑节点
                        // 如果successor是黑节点，他的右节点必然是红节点
                        // 将右节点设为黑色，可以抵消删除黑节点带来的黑高变化
                        let sr_ptr = sr.as_ptr();
                        unsafe {
                            (*sr_ptr).parent = Some(NonNull::new_unchecked(parent_ptr));
                            (*sr_ptr).color = RbColor::Black;
                        }
                    }
                    None => {
                        unsafe {
                            if let RbColor::Black = (*successor_ptr).color {
                                // 如果子节点不存在，且删除节点为黑色，则需要重新平衡
                                rebalance = Some(NonNull::new_unchecked(parent_ptr));
                            }
                        }
                    }
                }
            }
            (None, Some(right_node)) => {
                unsafe {
                    // 如果只有一个子树，该节点必然是黑色，子节点必然是红色
                    self.change_child(parent, NonNull::new_unchecked(node_ptr), right);

                    // 如果子节点存在，直接使用子节点替代，无需重新平衡
                    (*right_node.as_ptr()).color = RbColor::Black;
                }
            }
            (Some(left_node), None) => unsafe {
                self.change_child(parent, NonNull::new_unchecked(node_ptr), Some(left_node));
                (*left_node.as_ptr()).color = RbColor::Black;
            },
            (None, None) => {
                unsafe {
                    self.change_child(parent, NonNull::new_unchecked(node_ptr), left);
                    if let RbColor::Black = (*node_ptr).color {
                        // 如果子节点不存在，且删除节点为黑色，则需要重新平衡
                        rebalance = parent;
                    }
                }
            }
        }

        if let Some(rebalance_node) = rebalance {
            self.delete_fixup(rebalance_node);
        }
    }

    pub fn delete(&mut self, key: &T) {
        let mut node = &self.root;
        while let Some(n) = node {
            let node_ptr = n.as_ptr();
            let node_value = unsafe { &(*node_ptr).value };
            if key == node_value {
                // 找到节点，删除
                self.delete_node(node_ptr);
                let _ = unsafe { Box::from_raw(node_ptr) };
                return;
            } else if key < node_value {
                node = unsafe { &(*node_ptr).left };
            } else {
                node = unsafe { &(*node_ptr).right };
            }
        }
    }

    pub fn get(&self, key: &T) -> Option<&T> {
        let mut current = self.root;
        while let Some(node) = current {
            let node_ptr = node.as_ptr();
            let node_value = unsafe { &(*node_ptr).value };
            if key == node_value {
                return Some(node_value);
            } else if key < node_value {
                current = unsafe { (*node_ptr).left };
            } else {
                current = unsafe { (*node_ptr).right };
            }
        }
        None
    }

    pub fn enumerate(&self) -> Vec<&T> {
        let mut result = Vec::new();
        fn inorder<T: Ord>(node: Option<NonNull<RbNode<T>>>, result: &mut Vec<&T>) {
            if let Some(n) = node {
                unsafe {
                    inorder((*n.as_ptr()).left, result);
                    result.push(&(*n.as_ptr()).value);
                    inorder((*n.as_ptr()).right, result);
                }
            }
        }
        inorder(self.root, &mut result);
        result
    }
}

impl<T: Ord + Display + Default + Clone + Display> Display for RbTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let root = self.root;

        let mut matrix = vec![vec![]];
        fn fmt_node<T: Ord + Display + Clone>(
            node: Option<NonNull<RbNode<T>>>,
            depth: usize,
            matrix: &mut Vec<Vec<String>>,
        ) {
            if depth >= matrix.len() {
                matrix.push(vec![]);
            }
            if let Some(n) = node {
                let ptr = n.as_ptr();
                let str = (unsafe { (*ptr).value.clone() }).to_string();
                matrix[depth].push(str);
                fmt_node(unsafe { (*ptr).left }, depth + 1, matrix);
                fmt_node(unsafe { (*ptr).right }, depth + 1, matrix);
            } else {
                matrix[depth].push(String::from("."));
                return;
            }
        }
        fmt_node(root, 0, &mut matrix);

        write!(f, "Size: {}\n", self.size)?;
        let space = matrix.iter().map(|level| level.len()).max().unwrap_or(16) * 4;
        for (i, level) in matrix.iter().enumerate() {
            write!(f, "{}", " ".repeat(space - level.len() * 2))?;
            for node in level {
                write!(f, "{:3} ", node)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
