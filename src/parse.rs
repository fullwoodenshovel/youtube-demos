use std::{collections::{HashMap, VecDeque}, fmt::Debug};

#[cfg(target_family = "unix")]
use input_handler::InputHandler;
use macroquad::math::Vec2;
pub mod for_each;
pub mod visualise;
use crate::mat2::Mat2;

#[cfg(not(target_family = "unix"))]
pub struct InputHandler;

pub fn input(prompt: &str, handler: &mut InputHandler) -> Option<String> {
    // WASM: non-blocking — return None if JS hasn't submitted anything yet.
    #[cfg(target_arch = "wasm32")]
    { let _ = (prompt, handler); return crate::web::take_input(); }

    #[cfg(all(target_family = "unix", not(target_arch = "wasm32")))]
    return Some(match handler.readline(prompt) {
        Ok(input) => input,
        Err(err) => panic!("Error with input: {err}")
    });

    #[cfg(all(not(target_family = "unix"), not(target_arch = "wasm32")))]
    {
        let _ = handler;
        print!("{prompt}");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        Some(input)
    }
}

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

impl Debug for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Obj::Mat(mat2) => mat2.fmt(f),
            Obj::Vec(vec2) => vec2.fmt(f),
            Obj::Float(float) => float.fmt(f),
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Obj {
    fn to_string(&self) -> String {
        format!("{self:?}")
    }
}

#[derive(Clone, Debug)]
pub enum Ex {
    Mat(MatEx),
    Vec(VecEx),
    Float(FloatEx)
}

pub trait ExTrait: Clone {
    type Output;
    fn concrete(ex: Ex) -> Option<Self> where Self: Sized;
    fn concrete_err(ex: Ex) -> Result<Self, String> where Self: Sized {
        match Self::concrete(ex) {
            Some(value) => Ok(value),
            None => Err("Received wrong type in function use.".to_string())
        }
    }
    fn resolve(ex: &Self) -> Self::Output;
}

impl ExTrait for MatEx {
    type Output = Mat2;
    fn concrete(ex: Ex) -> Option<Self> {
        match ex {
            Ex::Mat(value) => Some(value),
            _ => None
        }
    }
    fn resolve(ex: &Self) -> Self::Output {
        resolve_mat(ex)
    }
}
impl ExTrait for VecEx {
    type Output = Vec2;
    fn concrete(ex: Ex) -> Option<Self> {
        match ex {
            Ex::Vec(value) => Some(value),
            _ => None
        }
    }
    fn resolve(ex: &Self) -> Self::Output {
        resolve_vec(ex)
    }
}
impl ExTrait for FloatEx {
    type Output = f32;
    fn concrete(ex: Ex) -> Option<Self> {
        match ex {
            Ex::Float(value) => Some(value),
            _ => None
        }
    }
    fn resolve(ex: &Self) -> Self::Output {
        resolve_float(ex)
    }
}

impl Ex {
    fn get_type(&self) -> &'static str {
        match self {
            Ex::Mat(_) => "Matrix",
            Ex::Vec(_) => "Vector",
            Ex::Float(_) => "Real",
        }
    }
}

#[derive(Debug)]
pub enum Line {
    Eval(Ex),
    SetVar(String, Ex),
    None,
}


#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    LBrace,
    RBrace,
    Float(f32),
    Mat,
    Vec,
    Comma,
    Add,
    Neg,
    Mul,
    Div,
    Pow,
    DotX,
    DotY,
    DotW,
    DotZ,
    DotA,
    DotB,
    DotC,
    DotD,
    DotI,
    DotJ,
    Left,
    Right,
    Top,
    Bottom,
    Hor,
    Inv,
    Vert,
    RotMat,
    RotVec,
    Cross,
    Det,
    Eq,
    Show,
    VarName(String),
}

fn make_exp(lhs: Ex, rhs: Ex, op: Token) -> Option<Ex> {
    let result = match lhs {
        Ex::Mat(lhs) => match rhs {
            Ex::Mat(rhs) => match op {
                Token::Add => Ex::Mat(MatEx::MatAdd(Box::new(lhs), Box::new(rhs))),
                Token::Neg => Ex::Mat(MatEx::MatSub(Box::new(lhs), Box::new(rhs))),
                Token::Mul => Ex::Mat(MatEx::MatMul(Box::new(lhs), Box::new(rhs))),
                Token::Div => Ex::Mat(MatEx::MatMul(Box::new(lhs), Box::new(MatEx::Inv(Box::new(rhs))))),
                _ => return None
            },
            Ex::Vec(rhs) => match op {
                Token::Mul => Ex::Vec(VecEx::VecMul(Box::new(lhs), Box::new(rhs))),
                _ => return None
            },
            Ex::Float(rhs) => match op {
                Token::Mul => Ex::Mat(MatEx::Mul(Box::new(rhs), Box::new(lhs))),
                Token::Div => Ex::Mat(MatEx::Div(Box::new(lhs), Box::new(rhs))),
                // Token::Pow => todo!(), // Raising matricies to float powers
                _ => return None
            },
        },
        Ex::Vec(lhs) => match rhs {
            Ex::Mat(_rhs) => return None,
            Ex::Vec(rhs) => match op {
                Token::Add => Ex::Vec(VecEx::VecAdd(Box::new(lhs), Box::new(rhs))),
                Token::Neg => Ex::Vec(VecEx::VecSub(Box::new(lhs), Box::new(rhs))),
                Token::Mul => Ex::Float(FloatEx::Dot(Box::new(lhs), Box::new(rhs))),
                Token::Cross => Ex::Float(FloatEx::Cross(Box::new(lhs), Box::new(rhs))),
                _ => return None
            },
            Ex::Float(rhs) => match op {
                Token::Mul => Ex::Vec(VecEx::Mul(Box::new(rhs), Box::new(lhs))),
                Token::Div => Ex::Vec(VecEx::Div(Box::new(lhs), Box::new(rhs))),
                _ => return None
            },
        },
        Ex::Float(lhs) => match rhs {
            Ex::Mat(rhs) => match op {
                Token::Mul => Ex::Mat(MatEx::Mul(Box::new(lhs), Box::new(rhs))),
                Token::Div => Ex::Mat(MatEx::Mul(Box::new(lhs), Box::new(MatEx::Inv(Box::new(rhs))))),
                _ => return None
            },
            Ex::Vec(rhs) => match op {
                Token::Mul => Ex::Vec(VecEx::Mul(Box::new(lhs), Box::new(rhs))),
                _ => return None
            },
            Ex::Float(rhs) => match op {
                Token::Add => Ex::Float(FloatEx::Add(Box::new(lhs), Box::new(rhs))),
                Token::Neg => Ex::Float(FloatEx::Sub(Box::new(lhs), Box::new(rhs))),
                Token::Mul => Ex::Float(FloatEx::Mul(Box::new(lhs), Box::new(rhs))),
                Token::Div => Ex::Float(FloatEx::Div(Box::new(lhs), Box::new(rhs))),
                Token::Pow => Ex::Float(FloatEx::Pow(Box::new(lhs), Box::new(rhs))),
                _ => return None
            },
        },
    };
    Some(result)
}


fn split_by_dot(iter: impl Iterator<Item = String>) -> impl Iterator<Item = String> {
    iter.flat_map(|s| {
        let mut result = Vec::new();
        let chars: Vec<char> = s.chars().collect();
        let mut last_split = 0;
        
        for i in 0..chars.len() {
            if chars[i] == '.' && i > 0 && !chars[i-1].is_ascii_digit() {
                let segment: String = chars[last_split..i].iter().collect();
                if !segment.is_empty() {
                    result.push(segment);
                }
                last_split = i;
            }
        }
        
        if last_split < chars.len() {
            let segment: String = chars[last_split..].iter().collect();
            if !segment.is_empty() {
                result.push(segment);
            }
        }
        
        result.into_iter()
    })
}


pub fn tokenise(inp: &str) -> Result<Vec<Token>, String> {
    if !inp.is_ascii() {
        return Err("Expression consists of non-ascii characters.".to_string())
    }
    
    let mut result = Vec::new();
    let inp = inp.trim().split(" ").map(|d| d.to_string());
    let inp = split_by_dot(inp);

    for inp in inp {
        let mut residual = String::new();
        
        fn append_residual(result: &mut Vec<Token>, residual: &str) {
            match &*residual.to_ascii_lowercase() {
                ".x" => result.push(Token::DotX),
                ".y" => result.push(Token::DotY),
                ".w" => result.push(Token::DotW),
                ".z" => result.push(Token::DotZ),
                ".a" => result.push(Token::DotA),
                ".b" => result.push(Token::DotB),
                ".c" => result.push(Token::DotC),
                ".d" => result.push(Token::DotD),
                ".i" => result.push(Token::DotI),
                ".j" => result.push(Token::DotJ),
                "left" => result.push(Token::Left),
                "right" => result.push(Token::Right),
                "top" => result.push(Token::Top),
                "bottom" => result.push(Token::Bottom),
                "hor" => result.push(Token::Hor),
                "inv" => result.push(Token::Inv),
                "vert" => result.push(Token::Vert),
                "mat" => result.push(Token::Mat),
                "vec" => result.push(Token::Vec),
                "rotmat" => result.push(Token::RotMat),
                "rotvec" => result.push(Token::RotVec),
                "det" => result.push(Token::Det),
                "show" => result.push(Token::Show),
                value => {
                    match value.parse() {
                        Ok(float) => result.push(Token::Float(float)),
                        Err(_) => result.push(Token::VarName(value.to_string())),
                    }
                }
            }
        }
        
        for char in inp.chars() {
            let mut added_to_residual = false;
            let mut push = None;
            match char {
                '(' => push = Some(Token::LBrace),
                ')' => push = Some(Token::RBrace),
                ',' => push = Some(Token::Comma),
                '+' => push = Some(Token::Add),
                '-' => push = Some(Token::Neg),
                '*' => push = Some(Token::Mul),
                '/' => push = Some(Token::Div),
                'X' => push = Some(Token::Cross),
                '=' => push = Some(Token::Eq),
                '^' => push = Some(Token::Pow),
                r => {
                    residual.push(r);
                    added_to_residual = true;
                }
            }

            if !added_to_residual && !residual.is_empty() {
                append_residual(&mut result, &residual);
                residual = String::new();
            }
            if let Some(value) = push {
                result.push(value);
            }
        }
        if !residual.is_empty() {
            append_residual(&mut result, &residual);
            residual = String::new();
        }
    }
    Ok(result)
}


pub struct Buffer<T> {
    data: VecDeque<T>,
}

impl<T: Debug> Buffer<T> {
    pub fn new(data: VecDeque<T>) -> Self {
        Self {
            data,
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.data.is_empty() {
            return None
        }
        Some(&self.data[0])
    }

    pub fn next(&mut self) -> Option<T> {
        self.data.pop_front()
    }
}

pub fn make_tree(vars: &HashMap<String, Obj>, tokens: Vec<Token>) -> Result<(Line, bool), String> {
    let mut tokens: VecDeque<Token> = tokens.into();
    let show = !tokens.is_empty() && Token::Show == tokens[0];
    if show {
        tokens.pop_front();
    }

    let result = if tokens.is_empty() {
        Line::None
    } else if tokens.len() >= 2 && let Token::VarName(name) = tokens[0].clone() && let Token::Eq = tokens[1] {
        tokens.pop_front();
        tokens.pop_front();
        let mut tokens = Buffer::new(tokens);
        let result = Line::SetVar(name, pratt_parse(vars, &mut tokens, 0)?);
        if let Some(token) = tokens.next() {
            return Err(format!("Expected end of expression, found token {token:?}"))
        }
        result
    } else {
        let mut tokens = Buffer::new(tokens);
        let result = Line::Eval(pratt_parse(vars, &mut tokens, 0)?);
        if let Some(token) = tokens.next() {
            return Err(format!("Expected end of expression, found token {token:?}"))
        }
        result
    };

    Ok((result, show))
}

fn end_of_ex(token: &Token) -> bool {
    matches!(token, Token::Comma | Token::RBrace)
}

fn binding_power(token: &Token) -> Option<(u8, u8)> {
    let result = match token {
        Token::Add => (1, 2),
        Token::Neg => (3, 4),
        Token::Mul => (5, 6),
        Token::Div => (5, 6),
        Token::Cross => (7, 8),
        Token::Pow => (10, 9),
        _ => return None
    };
    Some(result)
}

/// This supports functions with at least one argument
fn parse_func<const N: usize, T: ExTrait>(vars: &HashMap<String, Obj>, lexer: &mut Buffer<Token>) -> Result<[T; N], String> {
    let Some(token) = lexer.next() else {
        return Err("Unexpected end of expression.".to_string());
    };

    if Token::LBrace != token {
        return Err("You must have an open bracket before function use.".to_string())
    }

    let mut result = std::array::repeat(None);

    for n in result[..N-1].iter_mut() {
        *n = Some(T::concrete_err(pratt_parse(vars, lexer, 0)?)?);

        let Some(token) = lexer.next() else {
            return Err("Unexpected end of expression.".to_string());
        };

        if Token::Comma != token {
            return Err("You must have a comma after each argument in a function.".to_string())
        }
    }
    
    result[N - 1] = Some(T::concrete_err(pratt_parse(vars, lexer, 0)?)?);

    let Some(token) = lexer.next() else {
        return Err("No close bracket after function call".to_string())
    };

    if Token::RBrace != token {
        return Err(format!("Expected close bracket, got {token:?}"))
    }

    Ok(result.map(|d| d.unwrap()))
}

fn parse_func_boxed<const N: usize, T: ExTrait>(vars: &HashMap<String, Obj>, lexer: &mut Buffer<Token>) -> Result<[Box<T>; N], String> {
    Ok(parse_func(vars, lexer)?.map(|d| Box::new(d)))
}

fn pratt_parse(vars: &HashMap<String, Obj>, lexer: &mut Buffer<Token>, min_bp: u8) -> Result<Ex, String> {
    let Some(token) = lexer.next() else {
        return Err("Unexpected end of expression. This may be due to unclosed brackets.".to_string());
    };
    let mut lhs = match token {
        Token::Float(float) => Ex::Float(FloatEx::Literal(float)),
        Token::VarName(name) => match vars.get(&name) {
            None => return Err(format!("Variable `{name}` does not exist.")),
            Some(Obj::Float(float)) => Ex::Float(FloatEx::Literal(*float)),
            Some(Obj::Mat(mat)) => Ex::Mat(MatEx::Literal(*mat)),
            Some(Obj::Vec(vec)) => Ex::Vec(VecEx::Literal(*vec)),
        },
        Token::Left => {
            let [a] = parse_func_boxed(vars, lexer)?;
            Ex::Vec(VecEx::Left(a))
        },
        Token::Right => {
            let [a] = parse_func_boxed(vars, lexer)?;
            Ex::Vec(VecEx::Right(a))
        },
        Token::Top => {
            let [a] = parse_func_boxed(vars, lexer)?;
            Ex::Vec(VecEx::Top(a))
        },
        Token::Bottom => {
            let [a] = parse_func_boxed(vars, lexer)?;
            Ex::Vec(VecEx::Bottom(a))
        },
        Token::Hor => {
            let [a, b] = parse_func_boxed(vars, lexer)?;
            Ex::Mat(MatEx::Hor(a, b))
        },
        Token::Inv => {
            let [a] = parse_func_boxed(vars, lexer)?;
            Ex::Mat(MatEx::Inv(a))
        }
        Token::Vert => {
            let [a, b] = parse_func_boxed(vars, lexer)?;
            Ex::Mat(MatEx::Vert(a, b))
        },
        Token::LBrace => {
            let result = pratt_parse(vars, lexer, 0)?;

            let Some(token) = lexer.next() else {
                return Err("Expression ended without closing bracket".to_string())
            };

            if Token::RBrace != token {
                return Err("You are missing a close bracket".to_string())
            }

            result
        },
        Token::Mat => {
            let [a, b, c, d] = parse_func_boxed(vars, lexer)?;
            Ex::Mat(MatEx::New(a, b, c, d))
        },
        Token::Neg => {
            match pratt_parse(vars, lexer, 4)? {
                Ex::Mat(value) => Ex::Mat(MatEx::Neg(Box::new(value))),
                Ex::Vec(value) => Ex::Vec(VecEx::Neg(Box::new(value))),
                Ex::Float(value) => Ex::Float(FloatEx::Neg(Box::new(value))),
            }
        },
        Token::RotMat => {
            let [a] = parse_func_boxed(vars, lexer)?;
            Ex::Mat(MatEx::Rot(a))
        },
        Token::RotVec => {
            let [a] = parse_func_boxed(vars, lexer)?;
            Ex::Vec(VecEx::Rot(a))
        },
        Token::Vec => {
            let [a, b] = parse_func_boxed(vars, lexer)?;
            Ex::Vec(VecEx::New(a, b))
        },
        Token::Det => {
            let [a] = parse_func_boxed(vars, lexer)?;
            Ex::Float(FloatEx::Det(a))
        }
        other => return Err(format!("Unexpected token `{other:?}`."))
    };

    loop {
        let Some(op) = lexer.peek() else { break; };
        
        if end_of_ex(op) {
            break;
        }
        
        let Some((l_bp, r_bp)) = binding_power(op) else {
            let op = lexer.next().expect("Value here just checked by checking lexer.peek is Some");
            match lhs {
                Ex::Mat(ex) => match op {
                    Token::DotA | Token::DotX => lhs = Ex::Float(FloatEx::A(Box::new(ex))),
                    Token::DotB | Token::DotY => lhs = Ex::Float(FloatEx::B(Box::new(ex))),
                    Token::DotC | Token::DotW => lhs = Ex::Float(FloatEx::C(Box::new(ex))),
                    Token::DotD | Token::DotZ => lhs = Ex::Float(FloatEx::D(Box::new(ex))),
                    Token::DotI => lhs = Ex::Vec(VecEx::Left(Box::new(ex))),
                    Token::DotJ => lhs = Ex::Vec(VecEx::Right(Box::new(ex))),
                    _ => return Err(format!("Expected operation, DotA - DotD, DotX, DotZ, DotI or DotJ, got token `{op:?}`."))
                },
                Ex::Vec(ex) => match op {
                    Token::DotA | Token::DotX => lhs = Ex::Float(FloatEx::X(Box::new(ex))),
                    Token::DotB | Token::DotY => lhs = Ex::Float(FloatEx::Y(Box::new(ex))),
                    _ => return Err(format!("Expected operation, DotA, DotB, DotX or DotY, got token `{op:?}`."))
                },
                Ex::Float(_ex) => return Err(format!("Expected operation, got token `{op:?}`.")),
            }
            continue;
        };
        
        if l_bp < min_bp {
            break;
        }

        let op = lexer.next().expect("Value here just checked by checking lexer.peek is Some");

        let rhs = pratt_parse(vars, lexer, r_bp)?;
        let lhs_type = lhs.get_type();
        let rhs_type = rhs.get_type();
        let str_op = format!("{op:?}");
        lhs = match make_exp(lhs, rhs, op) {
            Some(lhs) => lhs,
            None => return Err(format!("You cannot perform the operation {} on a {} and a {}", str_op, lhs_type, rhs_type))
        }
    }
    Ok(lhs)

}

pub fn parse_exp(vars: &HashMap<String, Obj>, handler: &mut InputHandler) -> Option<(Line, bool)> {
    // The ? propagates None to the caller when there is no input yet (WASM idle).
    // On native platforms input() always returns Some, so ? never fires there.
    let raw = input("> ", handler)?;
    let tokenised = tokenise(&raw);
    match tokenised {
        Err(err) => {
            #[cfg(not(target_arch = "wasm32"))]
            eprintln!("{err}");
            #[cfg(target_arch = "wasm32")]
            crate::web::push_output(err);
        }
        Ok(tokens) => match make_tree(vars, tokens) {
            Ok(tree) => return Some(tree),
            Err(err) => {
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("{err}");
                #[cfg(target_arch = "wasm32")]
                crate::web::push_output(err);
            }
        }
    };
    
    None
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

#[allow(clippy::borrowed_box)] // This is because this is QOL, not functional.
fn resolve<T: ExTrait>(ex: &Box<T>) -> T::Output {
    T::resolve(ex.as_ref())
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