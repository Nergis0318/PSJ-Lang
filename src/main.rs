use std::{
    collections::HashMap,
    env,
    fmt::Display,
    io::{self, Read},
};

const HEADER: &str = "중국인";
const FOOTER: &str = "게이 박상진";

#[derive(Debug, Clone, PartialEq)]
struct Load {
    index: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct Term {
    load: Option<Load>,
    add: i32,
    input: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct Multiply {
    terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq)]
enum Statement {
    Assign {
        index: usize,
        value: Option<Multiply>,
    },
    PrintInt {
        value: Option<Multiply>,
    },
    PrintChar {
        codepoint: Option<Multiply>,
    },
    If {
        condition: Option<Multiply>,
        statement: Box<Statement>,
    },
    Goto {
        line: Multiply,
    },
    Exit {
        code: Option<Multiply>,
    },
}

#[derive(Debug)]
struct Program {
    statements: Vec<Option<Statement>>,
}

#[derive(Debug)]
enum RuntimeError {
    Parse(String),
    GotoOutOfRange,
    UnicodeOutOfRange,
    InputNotNumber,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "문법 오류: {msg}"),
            Self::GotoOutOfRange => write!(f, "\"상\" 명령의 줄 번호가 범위를 벗어났습니다."),
            Self::UnicodeOutOfRange => write!(f, "\"진ㅋ\"의 유니코드 값이 범위를 벗어났습니다."),
            Self::InputNotNumber => write!(f, "입력이 정수가 아닙니다."),
        }
    }
}

#[derive(Default)]
struct Parser;

impl Parser {
    fn parse_program(&self, source: &str) -> Result<Program, RuntimeError> {
        let normalized = source.replace('~', "\n");
        let lines: Vec<&str> = normalized
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();

        if lines.first().copied() != Some(HEADER) {
            return Err(RuntimeError::Parse(format!(
                "프로그램은 \"{HEADER}\" 로 시작해야 합니다."
            )));
        }
        if lines.last().copied() != Some(FOOTER) {
            return Err(RuntimeError::Parse(format!(
                "프로그램은 \"{FOOTER}\" 로 끝나야 합니다."
            )));
        }

        let mut statements = vec![None];
        for (idx, line) in lines[1..lines.len() - 1].iter().enumerate() {
            let stmt = if line.is_empty() {
                None
            } else {
                Some(
                    self.parse_statement(line)
                        .map_err(|e| RuntimeError::Parse(format!("{}번째 줄: {}", idx + 2, e)))?,
                )
            };
            statements.push(stmt);
        }
        statements.push(None);
        Ok(Program { statements })
    }

    fn parse_statement(&self, s: &str) -> Result<Statement, String> {
        if let Some(rest) = s.strip_prefix("게이") {
            let rest = rest.trim_start();
            let q = rest
                .find('?')
                .ok_or_else(|| "게이? 구문이 아닙니다".to_string())?;
            let cond_raw = &rest[..q];
            let nested_raw = rest[q + 1..].trim_start();
            let condition = if cond_raw.trim().is_empty() {
                None
            } else {
                Some(self.parse_multiply(cond_raw.trim())?)
            };
            let statement = Box::new(self.parse_statement(nested_raw)?);
            return Ok(Statement::If {
                condition,
                statement,
            });
        }

        if let Some(rest) = s.strip_prefix('상') {
            let line = self.parse_multiply(rest.trim_start())?;
            return Ok(Statement::Goto { line });
        }

        if let Some(rest) = s.strip_prefix("화이팅!") {
            let rest = rest.trim();
            let code = if rest.is_empty() {
                None
            } else {
                Some(self.parse_multiply(rest)?)
            };
            return Ok(Statement::Exit { code });
        }

        if s.starts_with('진') {
            if let Some(inner) = s.strip_prefix("진").and_then(|x| x.strip_suffix('!')) {
                let inner = inner.trim();
                return Ok(Statement::PrintInt {
                    value: if inner.is_empty() {
                        None
                    } else {
                        Some(self.parse_multiply(inner)?)
                    },
                });
            }
            if let Some(inner) = s.strip_prefix("진").and_then(|x| x.strip_suffix('ㅋ')) {
                let inner = inner.trim();
                return Ok(Statement::PrintChar {
                    codepoint: if inner.is_empty() {
                        None
                    } else {
                        Some(self.parse_multiply(inner)?)
                    },
                });
            }
        }

        let mut p_count = 0usize;
        for ch in s.chars() {
            if ch == '박' {
                p_count += 1;
            } else {
                break;
            }
        }
        if p_count > 0 {
            let rest = s.chars().skip(p_count).collect::<String>();
            if !rest.starts_with('상') {
                return Err("대입문은 박...상 형태여야 합니다".to_string());
            }
            let idx = p_count;
            let after = rest.chars().skip(1).collect::<String>();
            let after = after.trim();
            let value = if after.is_empty() {
                None
            } else {
                Some(self.parse_multiply(after)?)
            };
            return Ok(Statement::Assign { index: idx, value });
        }

        Err(format!("알 수 없는 구문: {s}"))
    }

    fn parse_multiply(&self, s: &str) -> Result<Multiply, String> {
        let terms: Result<Vec<_>, _> = s
            .split_whitespace()
            .map(|part| self.parse_term(part))
            .collect();
        let terms = terms?;
        if terms.is_empty() {
            return Err("곱셈식이 비어 있습니다".to_string());
        }
        Ok(Multiply { terms })
    }

    fn parse_term(&self, s: &str) -> Result<Term, String> {
        let mut idx = 0usize;
        let chars: Vec<char> = s.chars().collect();

        let mut load = None;
        let mut load_cnt = 0usize;
        while idx < chars.len() && chars[idx] == '박' {
            load_cnt += 1;
            idx += 1;
        }
        if load_cnt > 0 {
            load = Some(Load { index: load_cnt });
        }

        let mut add = 0i32;
        let mut input = 0usize;
        while idx < chars.len() {
            match chars[idx] {
                '.' => {
                    add += 1;
                    idx += 1;
                }
                ',' => {
                    add -= 1;
                    idx += 1;
                }
                '진' => {
                    if idx + 1 < chars.len() && chars[idx + 1] == '?' {
                        input += 1;
                        idx += 2;
                    } else {
                        return Err(format!("{s}: 진 다음에는 ?만 올 수 있습니다"));
                    }
                }
                _ => return Err(format!("{s}: 지원하지 않는 토큰이 있습니다")),
            }
        }

        if load.is_none() && add == 0 && input == 0 {
            return Err(format!("{s}: 빈 항입니다"));
        }

        Ok(Term { load, add, input })
    }
}

struct Context {
    len: usize,
    pc: usize,
    vars: HashMap<usize, i32>,
    exit: Option<i32>,
    input: Vec<i32>,
}

fn eval_multiply(ctx: &mut Context, multiply: &Multiply) -> Result<i32, RuntimeError> {
    let mut result = 1i32;
    for term in &multiply.terms {
        let load = term
            .load
            .as_ref()
            .map(|l| *ctx.vars.get(&l.index).unwrap_or(&0))
            .unwrap_or(0);
        let mut value = load.wrapping_add(term.add);
        for _ in 0..term.input {
            let next = ctx.input.pop().ok_or(RuntimeError::InputNotNumber)?;
            value = value.wrapping_add(next);
        }
        result = result.wrapping_mul(value);
    }
    Ok(result)
}

fn run_statement(
    ctx: &mut Context,
    stmt: &Statement,
    out: &mut String,
) -> Result<(), RuntimeError> {
    match stmt {
        Statement::Assign { index, value } => {
            let val = if let Some(m) = value {
                eval_multiply(ctx, m)?
            } else {
                0
            };
            ctx.vars.insert(*index, val);
        }
        Statement::PrintInt { value } => {
            let val = if let Some(m) = value {
                eval_multiply(ctx, m)?
            } else {
                0
            };
            out.push_str(&val.to_string());
        }
        Statement::PrintChar { codepoint } => {
            let ch = if let Some(m) = codepoint {
                let code = eval_multiply(ctx, m)?;
                let u = u32::try_from(code).map_err(|_| RuntimeError::UnicodeOutOfRange)?;
                char::from_u32(u).ok_or(RuntimeError::UnicodeOutOfRange)?
            } else {
                '\n'
            };
            out.push(ch);
        }
        Statement::If {
            condition,
            statement,
        } => {
            let cond = if let Some(m) = condition {
                eval_multiply(ctx, m)?
            } else {
                0
            };
            if cond == 0 {
                run_statement(ctx, statement, out)?;
            }
        }
        Statement::Goto { line } => {
            let target = eval_multiply(ctx, line)?;
            if target < 1 || target as usize > ctx.len {
                return Err(RuntimeError::GotoOutOfRange);
            }
            ctx.pc = target as usize - 1;
        }
        Statement::Exit { code } => {
            let code = if let Some(m) = code {
                eval_multiply(ctx, m)?
            } else {
                0
            };
            ctx.exit = Some(code);
        }
    }
    Ok(())
}

fn interpret(source: &str, input: &str) -> Result<(String, i32), RuntimeError> {
    let parser = Parser;
    let program = parser.parse_program(source)?;

    let mut nums: Vec<i32> = input
        .split_whitespace()
        .map(|t| t.parse::<i32>().map_err(|_| RuntimeError::InputNotNumber))
        .collect::<Result<Vec<_>, _>>()?;
    nums.reverse();

    let mut ctx = Context {
        len: program.statements.len(),
        pc: 0,
        vars: HashMap::new(),
        exit: None,
        input: nums,
    };
    let mut output = String::new();

    while ctx.pc < ctx.len {
        if let Some(stmt) = &program.statements[ctx.pc] {
            ctx.pc += 1;
            run_statement(&mut ctx, stmt, &mut output)?;
            if ctx.exit.is_some() {
                break;
            }
        } else {
            ctx.pc += 1;
        }
    }

    Ok((output, ctx.exit.unwrap_or(0)))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("사용법: psj-lang <source.psj>");
        std::process::exit(1);
    }

    let source = std::fs::read_to_string(&args[1]).expect("소스 파일을 읽을 수 없습니다");
    let mut stdin_buf = String::new();
    io::stdin().read_to_string(&mut stdin_buf).ok();

    match interpret(&source, &stdin_buf) {
        Ok((output, code)) => {
            print!("{}", output);
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_char_output() {
        let code = "중국인\n진........... ........ㅋ\n게이 박상진";
        let (out, rc) = interpret(code, "").unwrap();
        assert_eq!(out, "X");
        assert_eq!(rc, 0);
    }

    #[test]
    fn assign_and_print() {
        let code = "중국인\n박상..\n진박!\n게이 박상진";
        let (out, _) = interpret(code, "").unwrap();
        assert_eq!(out, "2");
    }

    #[test]
    fn if_and_goto() {
        let code = "중국인\n게이.?진..!\n상....\n진.....!\n화이팅!\n게이 박상진";
        let (out, _) = interpret(code, "").unwrap();
        assert_eq!(out, "5");
    }
}
