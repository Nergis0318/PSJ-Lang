#!/usr/bin/env python3
"""psj-lang (박상진 언어) Python implementation."""

from __future__ import annotations

import sys
from dataclasses import dataclass

HEADER = "중국인"
FOOTER = "게이 박상진"


@dataclass
class Load:
    index: int


@dataclass
class Term:
    load: Load | None
    add: int
    input_count: int


@dataclass
class Multiply:
    terms: list[Term]


@dataclass
class Assign:
    index: int
    value: Multiply | None


@dataclass
class PrintInt:
    value: Multiply | None


@dataclass
class PrintChar:
    value: Multiply | None


@dataclass
class IfStmt:
    condition: Multiply | None
    statement: object


@dataclass
class Goto:
    line: Multiply


@dataclass
class Exit:
    code: Multiply | None


Statement = Assign | PrintInt | PrintChar | IfStmt | Goto | Exit


class ParseError(Exception):
    pass


class RuntimeErrorPSJ(Exception):
    pass


class Parser:
    def parse_program(self, source: str) -> list[Statement | None]:
        normalized = source.replace("~", "\n")
        lines = [line.strip() for line in normalized.splitlines() if line.strip()]
        if not lines or lines[0] != HEADER:
            raise ParseError(f'프로그램은 "{HEADER}" 로 시작해야 합니다.')
        if lines[-1] != FOOTER:
            raise ParseError(f'프로그램은 "{FOOTER}" 로 끝나야 합니다.')

        statements: list[Statement | None] = [None]
        for line in lines[1:-1]:
            statements.append(self.parse_statement(line))
        statements.append(None)
        return statements

    def parse_statement(self, s: str) -> Statement:
        if s.startswith("게이"):
            rest = s[len("게이") :].lstrip()
            qidx = rest.find("?")
            if qidx == -1:
                raise ParseError("게이? 구문이 아닙니다")
            cond_raw = rest[:qidx].strip()
            body = rest[qidx + 1 :].lstrip()
            condition = None if not cond_raw else self.parse_multiply(cond_raw)
            return IfStmt(condition, self.parse_statement(body))

        if s.startswith("상"):
            return Goto(self.parse_multiply(s[1:].strip()))

        if s.startswith("화이팅!"):
            rest = s[len("화이팅!") :].strip()
            return Exit(None if not rest else self.parse_multiply(rest))

        if s.startswith("진") and s.endswith("!"):
            inside = s[1:-1].strip()
            return PrintInt(None if not inside else self.parse_multiply(inside))

        if s.startswith("진") and s.endswith("ㅋ"):
            inside = s[1:-1].strip()
            return PrintChar(None if not inside else self.parse_multiply(inside))

        p_count = 0
        for ch in s:
            if ch == "박":
                p_count += 1
            else:
                break
        if p_count > 0:
            rest = s[p_count:]
            if not rest.startswith("상"):
                raise ParseError("대입문은 박...상 형태여야 합니다")
            rhs = rest[1:].strip()
            value = None if not rhs else self.parse_multiply(rhs)
            return Assign(p_count, value)

        raise ParseError(f"알 수 없는 구문: {s}")

    def parse_multiply(self, s: str) -> Multiply:
        parts = s.split()
        if not parts:
            raise ParseError("곱셈식이 비어 있습니다")
        return Multiply([self.parse_term(part) for part in parts])

    def parse_term(self, s: str) -> Term:
        idx = 0
        load_count = 0
        while idx < len(s) and s[idx] == "박":
            load_count += 1
            idx += 1
        load = Load(load_count) if load_count else None

        add = 0
        input_count = 0
        while idx < len(s):
            ch = s[idx]
            if ch == ".":
                add += 1
                idx += 1
            elif ch == ",":
                add -= 1
                idx += 1
            elif ch == "진":
                if idx + 1 < len(s) and s[idx + 1] == "?":
                    input_count += 1
                    idx += 2
                else:
                    raise ParseError(f"{s}: 진 다음에는 ?만 올 수 있습니다")
            else:
                raise ParseError(f"{s}: 지원하지 않는 토큰이 있습니다")

        if load is None and add == 0 and input_count == 0:
            raise ParseError(f"{s}: 빈 항입니다")
        return Term(load, add, input_count)


def wrap_i32(v: int) -> int:
    return ((v + 2**31) % 2**32) - 2**31


def eval_multiply(m: Multiply, vars_: dict[int, int], inputs: list[int]) -> int:
    result = 1
    for term in m.terms:
        load = vars_.get(term.load.index, 0) if term.load else 0
        value = wrap_i32(load + term.add)
        for _ in range(term.input_count):
            if not inputs:
                raise RuntimeErrorPSJ("입력이 정수가 아닙니다.")
            value = wrap_i32(value + inputs.pop())
        result = wrap_i32(result * value)
    return result


def interpret(source: str, stdin_text: str) -> tuple[str, int]:
    parser = Parser()
    statements = parser.parse_program(source)
    try:
        nums = [int(x) for x in stdin_text.split()]
    except ValueError as exc:
        raise RuntimeErrorPSJ("입력이 정수가 아닙니다.") from exc
    nums.reverse()

    pc = 0
    out: list[str] = []
    vars_: dict[int, int] = {}
    exit_code: int | None = None

    while pc < len(statements):
        stmt = statements[pc]
        if stmt is None:
            pc += 1
            continue

        pc += 1
        if isinstance(stmt, Assign):
            val = 0 if stmt.value is None else eval_multiply(stmt.value, vars_, nums)
            vars_[stmt.index] = val
        elif isinstance(stmt, PrintInt):
            val = 0 if stmt.value is None else eval_multiply(stmt.value, vars_, nums)
            out.append(str(val))
        elif isinstance(stmt, PrintChar):
            if stmt.value is None:
                out.append("\n")
            else:
                code = eval_multiply(stmt.value, vars_, nums)
                if code < 0 or code > 0x10FFFF:
                    raise RuntimeErrorPSJ('"진ㅋ"의 유니코드 값이 범위를 벗어났습니다.')
                out.append(chr(code))
        elif isinstance(stmt, IfStmt):
            cond = 0 if stmt.condition is None else eval_multiply(stmt.condition, vars_, nums)
            if cond == 0:
                statements.insert(pc, stmt.statement)
        elif isinstance(stmt, Goto):
            line = eval_multiply(stmt.line, vars_, nums)
            if line < 1 or line > len(statements):
                raise RuntimeErrorPSJ('"상" 명령의 줄 번호가 범위를 벗어났습니다.')
            pc = line - 1
        elif isinstance(stmt, Exit):
            exit_code = 0 if stmt.code is None else eval_multiply(stmt.code, vars_, nums)
            break

    return "".join(out), 0 if exit_code is None else exit_code


def main() -> None:
    if len(sys.argv) < 2:
        print("사용법: python3 psj_lang.py <source.psj>", file=sys.stderr)
        raise SystemExit(1)

    source_path = sys.argv[1]
    with open(source_path, "r", encoding="utf-8") as f:
        source = f.read()
    stdin_text = sys.stdin.read()

    try:
        out, code = interpret(source, stdin_text)
        sys.stdout.write(out)
        raise SystemExit(code)
    except (ParseError, RuntimeErrorPSJ) as e:
        print(e, file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
