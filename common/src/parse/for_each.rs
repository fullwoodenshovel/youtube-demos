use std::{collections::VecDeque, fmt::Display};

use crate::parse::{resolve_mat, resolve_float, resolve_vec};

use super::{FloatEx, VecEx, MatEx, Ex, Obj};


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ExPointer<'a> {
    Mat(&'a MatEx),
    Float(&'a FloatEx),
    Vec(&'a VecEx),
}

impl<'a> ExPointer<'a> {
    pub fn resolve(self) -> Obj {
        match self {
            ExPointer::Mat(ex) => Obj::Mat(resolve_mat(ex)),
            ExPointer::Float(ex) => Obj::Float(resolve_float(ex)),
            ExPointer::Vec(ex) => Obj::Vec(resolve_vec(ex)),
        }
    }

    pub fn pointer_eq(self, other: ExPointer) -> bool {
        match self {
            ExPointer::Mat(ex) => if let ExPointer::Mat(ex1) = other {
                std::ptr::eq(ex, ex1)
            } else { false },
            ExPointer::Float(ex) => if let ExPointer::Float(ex1) = other {
                std::ptr::eq(ex, ex1)
            } else { false },
            ExPointer::Vec(ex) => if let ExPointer::Vec(ex1) = other {
                std::ptr::eq(ex, ex1)
            } else { false },
        }
    }

    pub fn from_ex(ex: &'a Ex) -> Self {
        match ex {
            Ex::Mat(ex) => Self::Mat(ex),
            Ex::Vec(ex) => Self::Vec(ex),
            Ex::Float(ex) => Self::Float(ex),
        }
    }
}

impl<'a> Display for ExPointer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            ExPointer::Mat(ex) => match ex {
                MatEx::MatMul(_, _) => "M * M",
                MatEx::MatAdd(_, _) => "M + M",
                MatEx::MatSub(_, _) => "M - M",
                MatEx::Neg(_) => "- M",
                MatEx::Mul(_, _) => "F * M",
                MatEx::Div(_, _) => "M / F",
                MatEx::Rot(_) => "Rot-M",
                MatEx::New(_, _, _, _) => "New-M",
                MatEx::Vert(_, _) => "Vert-M",
                MatEx::Hor(_, _) => "Hor-M",
                MatEx::Inv(_) => "Inv-M",
                MatEx::Literal(mat) => &format!(
                    "[{} {}; {} {}]",
                    format_min_chars(mat.a()),
                    format_min_chars(mat.b()),
                    format_min_chars(mat.c()),
                    format_min_chars(mat.d())
                ),
            },
            ExPointer::Float(ex) => match ex {
                FloatEx::A(_) => "M.a",
                FloatEx::B(_) => "M.b",
                FloatEx::C(_) => "M.c",
                FloatEx::D(_) => "M.d",
                FloatEx::X(_) => "V.x",
                FloatEx::Y(_) => "V.y",
                FloatEx::Mul(_, _) => "F * F",
                FloatEx::Div(_, _) => "F / F",
                FloatEx::Pow(_, _) => "F ^ F",
                FloatEx::Add(_, _) => "F + F",
                FloatEx::Sub(_, _) => "F - F",
                FloatEx::Neg(_) => "- F",
                FloatEx::Dot(_, _) => "V * V",
                FloatEx::Cross(_, _) => "V x V",
                FloatEx::Det(_) => "Det",
                FloatEx::Literal(float) => &format_min_chars(*float),
            },
            ExPointer::Vec(ex) => match ex {
                VecEx::VecMul(_, _) => "M * V",
                VecEx::VecAdd(_, _) => "V + V",
                VecEx::VecSub(_, _) => "V - V",
                VecEx::Neg(_) => "- V",
                VecEx::Mul(_, _) => "F * V",
                VecEx::Div(_, _) => "V / F",
                VecEx::Rot(_) => "Rot-V",
                VecEx::Left(_) => "Left",
                VecEx::Right(_) => "Right",
                VecEx::Top(_) => "Top",
                VecEx::Bottom(_) => "Bottom",
                VecEx::New(_, _) => "New-V",
                VecEx::Literal(vec) => &format!("({} {})",
                    format_min_chars(vec.x),
                    format_min_chars(vec.y)
                ),
            },
        };
        write!(f, "{}", result)
    }
}

/// Toggling reverse_children and children_first is equivalent to reversing the order of the returned Vec.
pub fn for_each(ex: &Ex, children_first: bool, reverse_children: bool) -> VecDeque<(ExPointer<'_>, usize)> {
    for_each_pointer(ExPointer::from_ex(ex), children_first, reverse_children)
}

pub fn for_each_pointer<'a>(ex: ExPointer<'a>, children_first: bool, reverse_children: bool) -> VecDeque<(ExPointer<'a>, usize)> {
    let mut result = match ex {
        ExPointer::Mat(ex) => for_each_mat(ex, 0, children_first ^ reverse_children),
        ExPointer::Vec(ex) => for_each_vec(ex, 0, children_first ^ reverse_children),
        ExPointer::Float(ex) => for_each_float(ex, 0, children_first ^ reverse_children),
    };
    if reverse_children {
        result.make_contiguous().reverse();
    }
    result
}

pub fn resolve_indexed(index: usize, ex: &Ex) -> Obj {
    for_each(ex, true, false)[index].0.resolve()
}

pub fn for_each_mat<'a>(ex: &'a MatEx, depth: usize, children_first: bool) -> VecDeque<(ExPointer<'a>, usize)> {
    let next_depth = depth + 1;
    let mut result = VecDeque::new();
    match ex {
        MatEx::MatMul(ex, ex1) |
        MatEx::MatAdd(ex, ex1) |
        MatEx::MatSub(ex, ex1) => {
            result.append(&mut for_each_mat(ex, next_depth, children_first));
            result.append(&mut for_each_mat(ex1, next_depth, children_first));
        },
        MatEx::Mul(ex, ex1) => {
            result.append(&mut for_each_float(ex, next_depth, children_first));
            result.append(&mut for_each_mat(ex1, next_depth, children_first));
        }
        MatEx::Neg(ex) |
        MatEx::Inv(ex) => result.append(&mut for_each_mat(ex, next_depth, children_first)),
        MatEx::Div(ex, ex1) => {
            result.append(&mut for_each_mat(ex, next_depth, children_first));
            result.append(&mut for_each_float(ex1, next_depth, children_first));
        },
        MatEx::Rot(ex) => result.append(&mut for_each_float(ex, next_depth, children_first)),
        MatEx::New(ex, ex1, ex2, ex3) => {
            result.append(&mut for_each_float(ex, next_depth, children_first));
            result.append(&mut for_each_float(ex1, next_depth, children_first));
            result.append(&mut for_each_float(ex2, next_depth, children_first));
            result.append(&mut for_each_float(ex3, next_depth, children_first));
        },
        MatEx::Vert(ex, ex1) |
        MatEx::Hor(ex, ex1) => {
            result.append(&mut for_each_vec(ex, next_depth, children_first));
            result.append(&mut for_each_vec(ex1, next_depth, children_first));
        },
        MatEx::Literal(_) => (),
        
    };
    if children_first {
        result.push_back((ExPointer::Mat(ex), depth));
    } else {
        result.push_front((ExPointer::Mat(ex), depth));
    }
    result
}

pub fn for_each_float<'a>(ex: &'a FloatEx, depth: usize, children_first: bool) -> VecDeque<(ExPointer<'a>, usize)> {
    let next_depth = depth + 1;
    let mut result = VecDeque::new();
    match ex {
        FloatEx::A(ex) |
        FloatEx::B(ex) |
        FloatEx::C(ex) |
        FloatEx::D(ex) => result.append(&mut for_each_mat(ex, next_depth, children_first)),
        FloatEx::X(ex) |
        FloatEx::Y(ex) => result.append(&mut for_each_vec(ex, next_depth, children_first)),
        FloatEx::Mul(ex, ex1) |
        FloatEx::Div(ex, ex1) |
        FloatEx::Pow(ex, ex1) |
        FloatEx::Add(ex, ex1) |
        FloatEx::Sub(ex, ex1) => {
            result.append(&mut for_each_float(ex, next_depth, children_first));
            result.append(&mut for_each_float(ex1, next_depth, children_first));
        },
        FloatEx::Neg(ex) => result.append(&mut for_each_float(ex, next_depth, children_first)),
        FloatEx::Dot(ex, ex1) |
        FloatEx::Cross(ex, ex1) => {
            result.append(&mut for_each_vec(ex, next_depth, children_first));
            result.append(&mut for_each_vec(ex1, next_depth, children_first));
        },
        FloatEx::Det(ex) => result.append(&mut for_each_mat(ex, next_depth, children_first)),
        FloatEx::Literal(_) => (),
    };
    if children_first {
        result.push_back((ExPointer::Float(ex), depth));
    } else {
        result.push_front((ExPointer::Float(ex), depth));
    }
    result
}

pub fn for_each_vec<'a>(ex: &'a VecEx, depth: usize, children_first: bool) -> VecDeque<(ExPointer<'a>, usize)> {
    let next_depth = depth + 1;
    let mut result = VecDeque::new();
    match ex {
        VecEx::VecMul(ex, ex1) => {
            result.append(&mut for_each_mat(ex, next_depth, children_first));
            result.append(&mut for_each_vec(ex1, next_depth, children_first));
        },
        VecEx::VecAdd(ex, ex1) |
        VecEx::VecSub(ex, ex1) => {
            result.append(&mut for_each_vec(ex, next_depth, children_first));
            result.append(&mut for_each_vec(ex1, next_depth, children_first));
        },
        VecEx::Neg(ex) => result.append(&mut for_each_vec(ex, next_depth, children_first)),
        VecEx::Mul(ex, ex1) |
        VecEx::Div(ex1, ex) => {
            result.append(&mut for_each_float(ex, next_depth, children_first));
            result.append(&mut for_each_vec(ex1, next_depth, children_first));
        },
        VecEx::Rot(ex) => result.append(&mut for_each_float(ex, next_depth, children_first)),
        VecEx::Left(ex) |
        VecEx::Right(ex) |
        VecEx::Top(ex) |
        VecEx::Bottom(ex) => result.append(&mut for_each_mat(ex, next_depth, children_first)),
        VecEx::New(ex, ex1) => {
            result.append(&mut for_each_float(ex, next_depth, children_first));
            result.append(&mut for_each_float(ex1, next_depth, children_first));
        },
        VecEx::Literal(_) => (),
    };
    if children_first {
        result.push_back((ExPointer::Vec(ex), depth));
    } else {
        result.push_front((ExPointer::Vec(ex), depth));
    }
    result
}

fn format_min_chars(x: f32) -> String {
    if x == 0.0 {
        return "0".to_string();
    }

    let abs = x.abs();
    let exp = abs.log10().floor();
    let scale = 10f32.powf(1.0 - exp); // for 2 significant figures
    let rounded = (x * scale).round() / scale;
    let mut fixed = if x < 10.0 {
        format!("{:.7}", rounded)
    } else {
        format!("{:.0}", x)
    };

    // Fixed notation (trim trailing zeros and dot)
    while fixed.contains('.') && fixed.ends_with('0') {
        fixed.pop();
    }
    if fixed.ends_with('.') {
        fixed.pop();
    }

    // Scientific notation with 1 digit after decimal (2 s.f.)
    let sci = format!("{:.1e}", rounded);

    if sci.len() < fixed.len() {
        sci
    } else {
        fixed
    }
}