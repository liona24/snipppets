import json
import re
from itertools import chain
import ast
from collections import deque
from typing import List, Union, Dict, Any

__all___ = [ "explore" ]

VAR_PATTERN = re.compile(r"\$([^$]*(?:\$\$[^$]*)*)\$")

_Empty = object()


class _Scope(object):
    def __init__(self, name, parent: '_Scope' = None):
        self._name = name
        self._parent = parent
        self._data = {}

    @property
    def data(self):
        return self._data

    def __iter__(self):
        if self._parent is not None:
            it = iter(self._parent)
        else:
            it = iter([])

        return chain([self], it)

    def __hash__(self):
        return hash(self._name)

    def __getitem__(self, key):
        return self._data[key]

    def __contains__(self, key):
        return key in self._data

    def __setitem__(self, key, value):
        self._data[key] = value

    def add_child_scope(self, name):
        scp = _Scope(name, self)
        self._data[name] = scp
        return scp

    def clear(self):
        for key in self._data:
            self._data[key] = _Empty

    def find(self, key):
        for scope in self:
            if key in scope:
                return scope

        return None

    def __repr__(self):
        parent = None
        if self._parent is not None:
            parent = self._parent._name
        return f"<_Scope {self._name} data={self._data} parent={parent}>"


class _Dependency(object):
    def __init__(self, scope: _Scope, key: str):
        self.scope = scope
        self.key = key

    @property
    def is_resolved(self) -> bool:
        value = self.scope[self.key]
        if isinstance(value, _Scope):
            for value in value.data.items():
                if isinstance(value, (_Scope, _LazyExpr, _MixedExpr)) or value is _Empty:
                    return False
            return True
        elif isinstance(value, (_LazyExpr, _MixedExpr)) or value is _Empty:
            return False
        else:
            return True


def resolve_named_variables(ast_, scope: _Scope):
    class NameResolver(ast.NodeTransformer):
        def visit_Name(self, node):
            if not isinstance(node.ctx, ast.Load):
                raise TypeError("Unsupported node!")

            rv = scope.find(node.id)
            if rv is None:
                return None

            value = rv[node.id]

            if isinstance(value, (_LazyExpr, _MixedExpr)):
                value = value.evaluate()

            return ast.copy_location(ast.Constant(value=value, kind=None), node)

    return NameResolver().visit(ast_)


def find_dependencies(ast_, scope: _Scope) -> List[_Dependency]:
    deps = []

    class Visitor(ast.NodeVisitor):
        def visit_Name(self, node):
            if not isinstance(node.ctx, ast.Load):
                return

            dep = scope.find(node.id)
            deps.append(_Dependency(dep, node.id))

    Visitor().visit(ast_)

    return deps


def product(*iterables):
    """A non-caching version of itertools.product

    It appears itertools.product does construct an iterator for all the iterables once
    and then caches the result rather than constructing a new iterator for each
    iterator each time.
    We need the re-evaluation to traverse levels of ``_ParameterCollection``s lazily.

    :yield: ``tuple`` of elements of length len(iterables)
    :rtype: None
    """
    stack = [ iter(it) for it in iterables ]
    level = len(stack) - 1

    values = [ next(it) for it in stack ]
    yield tuple(values)

    while level >= 0:
        try:
            values[level] = next(stack[level])
        except StopIteration:
            level -= 1
            continue

        while level < len(stack) - 1:
            level += 1
            stack[level] = iter(iterables[level])
            values[level] = next(stack[level])

        yield tuple(values)


class _LazyExpr(object):
    def __init__(self, expr: str, scope: _Scope):
        self.expr = expr
        self.scope = scope

        ast_ = ast.parse(expr, mode="eval")
        self.dependencies = find_dependencies(ast_, scope)

        for dep in self.dependencies:
            if dep.scope is None:
                raise NameError(f"'{dep.key}' is not defined (in expression '{expr}')")

    @property
    def local_dependencies(self):
        return filter(lambda dep: dep[0] is self.scope, self.dependencies)

    def evaluate(self) -> Any:
        expr = ast.parse(self.expr, mode="eval")
        try:
            expr = resolve_named_variables(expr, self.scope)
            rv = eval(compile(expr, self.expr, "eval"))
        except TypeError:
            raise ValueError(f"Invalid expression '{self.expr}'")

        return rv


class _MixedExpr(object):
    def __init__(self, parts: List[Union[str, _LazyExpr]]):
        self._parts = parts

        self.dependencies = []
        for part in self._parts:
            if isinstance(part, _LazyExpr):
                self.dependencies.extend(part.dependencies)

    @staticmethod
    def _resolve_part(part: Union[str, _LazyExpr]) -> str:
        if isinstance(part, str):
            return part

        assert isinstance(part, _LazyExpr)
        return str(part.evaluate())

    def evaluate(self) -> str:
        return ''.join(map(self._resolve_part, self._parts))


class _ParameterCollection(object):
    def __init__(self, keys, values: List[List[Any]], scope: _Scope):
        self._iterator = product(*values)
        self._keys = keys

        self._scope = scope

    def __iter__(self):
        return self

    def __next__(self):
        values = next(self._iterator)

        self._scope.clear()

        for key, value in zip(self._keys, values):
            self._scope[key] = value

        return self._scope


def _evaluate_lazies(scp: _Scope) -> Dict:
    # this is currently a very simple brute force search. Works fine here though I suppose
    unresolved_deps = True

    while unresolved_deps:
        q = deque([ scp ])

        unresolved_deps = False
        infinity_loop_detector = True

        while len(q) > 0:
            top = q.popleft()

            for key, value in top.data.items():
                if isinstance(value, _Scope):
                    q.append(value)
                elif isinstance(value, (_MixedExpr, _LazyExpr)):
                    if all(dep.is_resolved for dep in value.dependencies):
                        infinity_loop_detector = False
                        top[key] = value.evaluate()
                    else:
                        unresolved_deps = True

        if infinity_loop_detector and unresolved_deps:
            raise ValueError("Circular dependencies detected!")

    ans = {}
    q = deque([ (scp, ans) ])
    while len(q) > 0:
        top, target = q.popleft()
        for key, value in top.data.items():
            if isinstance(value, _Scope):
                target[key] = {}
                q.append((value, target[key]))
            else:
                target[key] = value

    return ans


def _resolve_variables(element, scope: _Scope):
    if isinstance(element, str):
        parts = []
        last = 0
        for match in VAR_PATTERN.finditer(element):
            parts.append(element[last:match.start()])
            parts.append(_LazyExpr(match.group(1), scope))

            last = match.end()

        if len(parts) == 0:
            return element

        if last != len(element):
            parts.append(element[last:])

        parts = list(filter(None, parts))

        # We allow _LazyExpr explicitly because it may preserve type information
        if len(parts) == 1:
            return parts[0]

        return _MixedExpr(parts)

    if isinstance(element, list):
        return [ _resolve_variables(el, scope) for el in element ]

    return element


def _generate_recursively(dictionary: Dict, local_scope: _Scope):
    values = []
    keys = []

    # We have to set up the scopes pre-emptvely in order to allow dependencies to
    # resolve
    for key in dictionary:
        keys.append(key)
        local_scope[key] = _Empty

    for key in dictionary:
        if isinstance(dictionary[key], dict):
            values.append(_generate_recursively(
                dictionary[key],
                local_scope.add_child_scope(key)
            ))
        else:
            possible_values = _resolve_variables(dictionary[key], local_scope)
            if isinstance(possible_values, list):
                values.append(possible_values)
            else:
                values.append([ possible_values ])

    return _ParameterCollection(
        keys, values,
        local_scope
    )


def explore(file_name_or_dict):
    if isinstance(file_name_or_dict, str):
        with open(file_name_or_dict, "r") as f:
            inp = json.load(f)
    else:
        inp = file_name_or_dict

    scope = _Scope("__root__")
    for obj in _generate_recursively(inp, scope):
        yield _evaluate_lazies(obj)
