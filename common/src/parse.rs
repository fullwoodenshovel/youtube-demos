use std::fmt::{Debug, Display};

use macroquad::math::Vec2;
pub mod for_each;
pub mod visualise;
use crate::mat2::Mat2;

#[derive(Clone, Debug, PartialEq)]
pub enum MatEx {
    MatMul(Box<MatEx>, Box<MatEx>),
    MatAdd(Box<MatEx>, Box<MatEx>),
    MatSub(Box<MatEx>, Box<MatEx>),
    Neg(Box<MatEx>),
    Mul(Box<FloatEx>, Box<MatEx>),
    Div(Box<MatEx>, Box<FloatEx>),
    Rot(Box<FloatEx>),
    New(Box<FloatEx>, Box<FloatEx>, Box<FloatEx>, Box<FloatEx>),
    Vert(Box<VecEx>, Box<VecEx>),
    Hor(Box<VecEx>, Box<VecEx>),
    Inv(Box<MatEx>),
    Literal(Mat2)
}

#[derive(Clone, Debug, PartialEq)]
pub enum VecEx {
    VecMul(Box<MatEx>, Box<VecEx>),
    VecAdd(Box<VecEx>, Box<VecEx>),
    VecSub(Box<VecEx>, Box<VecEx>),
    Neg(Box<VecEx>),
    Mul(Box<FloatEx>, Box<VecEx>),
    Div(Box<VecEx>, Box<FloatEx>),
    Rot(Box<FloatEx>),
    Left(Box<MatEx>),
    Right(Box<MatEx>),
    Top(Box<MatEx>),
    Bottom(Box<MatEx>),
    New(Box<FloatEx>, Box<FloatEx>),
    Literal(Vec2)
}

#[derive(Clone, Debug, PartialEq)]
pub enum FloatEx {
    A(Box<MatEx>),
    B(Box<MatEx>),
    C(Box<MatEx>),
    D(Box<MatEx>),
    X(Box<VecEx>),
    Y(Box<VecEx>),
    Mul(Box<FloatEx>, Box<FloatEx>),
    Div(Box<FloatEx>, Box<FloatEx>),
    Pow(Box<FloatEx>, Box<FloatEx>),
    Add(Box<FloatEx>, Box<FloatEx>),
    Sub(Box<FloatEx>, Box<FloatEx>),
    Neg(Box<FloatEx>),
    Dot(Box<VecEx>, Box<VecEx>),
    Cross(Box<VecEx>, Box<VecEx>),
    Det(Box<MatEx>),
    Literal(f32)
}

#[derive(Clone, Copy)]
pub enum Obj {
    Mat(Mat2),
    Vec(Vec2),
    Float(f32)
}

impl Display for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Obj::Mat(mat2) => write!(f, "{}", mat2),
            Obj::Vec(vec2) => write!(f, "{}", vec2),
            Obj::Float(float) => write!(f, "{}", float),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Ex {
    Mat(MatEx),
    Vec(VecEx),
    Float(FloatEx)
}

trait ExTrait: Clone {
    type Output;
    fn resolve(ex: &Self) -> Self::Output;
}

impl ExTrait for MatEx {
    type Output = Mat2;
    fn resolve(ex: &Self) -> Self::Output {
        resolve_mat(ex)
    }
}
impl ExTrait for VecEx {
    type Output = Vec2;
    fn resolve(ex: &Self) -> Self::Output {
        resolve_vec(ex)
    }
}
impl ExTrait for FloatEx {
    type Output = f32;
    fn resolve(ex: &Self) -> Self::Output {
        resolve_float(ex)
    }
}

impl Ex {
    pub fn get_type(&self) -> &'static str {
        match self {
            Ex::Mat(_) => "Matrix",
            Ex::Vec(_) => "Vector",
            Ex::Float(_) => "Real",
        }
    }
}

pub fn resolve_ex(ex: &Ex) -> Obj {
    match ex {
        Ex::Float(ex) => Obj::Float(resolve_float(ex)),
        Ex::Mat(ex) => Obj::Mat(resolve_mat(ex)),
        Ex::Vec(ex) => Obj::Vec(resolve_vec(ex)),
    }
}

pub fn resolve_float(ex: &FloatEx) -> f32 {
    match ex {
        FloatEx::A(ex) => resolve(ex).a(),
        FloatEx::B(ex) => resolve(ex).b(),
        FloatEx::C(ex) => resolve(ex).c(),
        FloatEx::D(ex) => resolve(ex).c(),
        FloatEx::X(ex) => resolve(ex).x,
        FloatEx::Y(ex) => resolve(ex).y,
        FloatEx::Mul(ex, ex1) => resolve(ex) * resolve(ex1),
        FloatEx::Div(ex, ex1) => resolve(ex) / resolve(ex1),
        FloatEx::Pow(ex, ex1) => resolve(ex).powf(resolve(ex1)),
        FloatEx::Add(ex, ex1) => resolve(ex) + resolve(ex1),
        FloatEx::Sub(ex, ex1) => resolve(ex) - resolve(ex1),
        FloatEx::Neg(ex) => -resolve(ex),
        FloatEx::Dot(ex, ex1) => resolve(ex).dot(resolve(ex1)),
        FloatEx::Cross(ex, ex1) => {let a = resolve(ex); let b = resolve(ex1); a.x * b.y - a.y * b.x},
        FloatEx::Det(ex) => resolve(ex).det(),
        FloatEx::Literal(float) => *float,
    }
}

pub fn resolve_mat(ex: &MatEx) -> Mat2 {
    match ex {
        MatEx::MatMul(ex, ex1) => resolve(ex) * resolve(ex1),
        MatEx::MatAdd(ex, ex1) => resolve(ex) + resolve(ex1),
        MatEx::MatSub(ex, ex1) => resolve(ex) - resolve(ex1),
        MatEx::Neg(ex) => - resolve(ex),
        MatEx::Mul(ex, ex1) => resolve(ex) * resolve(ex1),
        MatEx::Div(ex, ex1) => resolve(ex) / resolve(ex1),
        MatEx::Rot(ex) => Mat2::rotation(resolve(ex)),
        MatEx::New(ex, ex1, ex2, ex3) => Mat2::new(resolve(ex), resolve(ex1), resolve(ex2), resolve(ex3)),
        MatEx::Vert(ex, ex1) => {let a = resolve(ex); let b = resolve(ex1); Mat2::new(a.x, b.x, a.y, b.y)},
        MatEx::Hor(ex, ex1) => {let a = resolve(ex); let b = resolve(ex1); Mat2::new(a.x, a.y, b.x, b.y)},
        MatEx::Inv(ex) => resolve(ex).inv(),
        MatEx::Literal(mat) => *mat,
    }
}

pub fn resolve_vec(ex: &VecEx) -> Vec2 {
    match ex {
        VecEx::VecMul(ex, ex1) => resolve(ex) * resolve(ex1),
        VecEx::VecAdd(ex, ex1) => resolve(ex) + resolve(ex1),
        VecEx::VecSub(ex, ex1) => resolve(ex) - resolve(ex1),
        VecEx::Neg(ex) => -resolve(ex),
        VecEx::Mul(ex, ex1) => resolve(ex) * resolve(ex1),
        VecEx::Div(ex, ex1) => resolve(ex) / resolve(ex1),
        VecEx::Rot(ex) => Vec2::from_angle(resolve(ex)),
        VecEx::Left(ex) => {let mat = resolve(ex); Vec2::new(mat.a(), mat.c())},
        VecEx::Right(ex) => {let mat = resolve(ex); Vec2::new(mat.b(), mat.d())},
        VecEx::Top(ex) => {let mat = resolve(ex); Vec2::new(mat.a(), mat.b())},
        VecEx::Bottom(ex) => {let mat = resolve(ex); Vec2::new(mat.c(), mat.d())},
        VecEx::New(ex, ex1) => Vec2::new(resolve(ex), resolve(ex1)),
        VecEx::Literal(vec) => *vec,
    }
}

#[allow(clippy::borrowed_box)] // This is because this is QOL, not functional.
fn resolve<T: ExTrait>(ex: &Box<T>) -> T::Output {
    T::resolve(ex.as_ref())
}