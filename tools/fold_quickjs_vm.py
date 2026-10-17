#!/usr/bin/env python3
"""Fold the translated C VM CFG without changing opcode bodies or shared labels.

Single-incoming-edge blocks execute in their predecessor; bounded small
shared tails expose the same direct branches as C. Self backedges become local
Rust loops. Other exits retain a dispatcher. --direct-opcodes makes opcode
handlers the direct match arms and reserves values above 255 for C control labels.
Run on the original numeric-label lowering, not on previously folded output.
Address-taken locals remain function-scoped; single-region scalar locals can
be declared in that region, with Rust checking assignment before use.
"""
import argparse, collections, heapq, json, pathlib, re


def mask(source):
    out = list(source)
    i = 0
    while i < len(source):
        end = None
        if source.startswith('//', i):
            end = source.find('\n', i)
            if end < 0: end = len(source)
        elif source.startswith('/*', i):
            depth, j = 1, i + 2
            while depth:
                if source.startswith('/*', j): depth += 1; j += 2
                elif source.startswith('*/', j): depth -= 1; j += 2
                else: j += 1
                if j > len(source): raise ValueError('unterminated comment')
            end = j
        else:
            raw = re.match(r'(?:br|cr|r)(#{0,16})"', source[i:])
            if raw:
                closing = '"' + raw[1]
                j = source.find(closing, i + raw.end())
                if j < 0: raise ValueError('unterminated raw string')
                end = j + len(closing)
            elif source[i] == '"':
                j = i + 1
                while j < len(source):
                    if source[j] == '\\': j += 2
                    elif source[j] == '"': j += 1; break
                    else: j += 1
                end = j
            elif source[i] == "'":
                char = re.match(r"'(?:\\.|[^\\'\n])'", source[i:])
                if char: end = i + char.end()
        if end is not None:
            for j in range(i, end):
                if out[j] != '\n': out[j] = ' '
            i = end
        else: i += 1
    return ''.join(out)


def close(text, start, opening='{', closing='}'):
    assert text[start] == opening
    depth = 1
    for i in range(start + 1, len(text)):
        if text[i] == opening: depth += 1
        elif text[i] == closing:
            depth -= 1
            if depth == 0: return i
    raise ValueError('unbalanced source')


def match_arms(text, start):
    cursor, parens, brackets = start, 0, 0
    while cursor < len(text):
        char = text[cursor]
        if char == '(': parens += 1
        elif char == ')': parens -= 1
        elif char == '[': brackets += 1
        elif char == ']': brackets -= 1
        elif char == '{' and parens == brackets == 0:
            end = close(text, cursor)
            if '=>' in text[cursor:end]: return cursor, end
            cursor = end
        cursor += 1
    raise ValueError('missing match arm list')


def transitions(body, mapping):
    code, changes = mask(body), []
    for m in re.finditer(r'\bvm_block\s*=(?!=)\s*', code):
        number = re.match(r'(\d+)\s*;', code[m.end():])
        if number:
            a = m.end() + number.start(1); b = m.end() + number.end(1)
            changes.append((a, b, mapping[int(number[1])]))
        else:
            assert code[m.end():].startswith('match')
            a, b = match_arms(code, m.end() + 5)
            for target in re.finditer(r'=>\s*(\d+)\b', code[a:b + 1]):
                lo, hi = a + target.start(1), a + target.end(1)
                changes.append((lo, hi, mapping[int(target[1])]))
    return replace(body, changes)


def direct_dispatch(prefix, nodes):
    code = mask(prefix)
    fetch = re.search(r'\bvm_block\s*=(?!=)\s*match\b', code)
    assert fetch
    a, b = match_arms(code, fetch.end())
    pairs = re.findall(r'\b(OP_\w+)\s*=>\s*(\d+)', code[a:b + 1])
    dispatch = int(re.search(r'while vm_block == (\d+)', code)[1])
    invalid = int(re.search(r'\b_\s*=>\s*(\d+)', code[a:b + 1])[1])
    groups = collections.defaultdict(list)
    for opcode, target in pairs: groups[int(target)].append(opcode)
    assert groups.pop(dispatch) == ['OP_nop']
    assert set(groups) <= set(nodes)
    auxiliary = sorted((set(nodes) - set(groups)) | {dispatch})
    mapping = {n: ops[0] for n, ops in groups.items()}
    mapping.update({n:str(256 + i) for i, n in enumerate(auxiliary)})
    assert 256 + len(auxiliary) < 65536
    labels = {n: ' | '.join(groups[n]) if n in groups else mapping[n] for n in nodes}
    nodes = {n:transitions(body, mapping) for n, body in nodes.items()}
    declaration = re.search(r'let mut vm_block: usize = (\d+);', code)
    condition = re.search(r'while vm_block == (\d+)', code)
    end = b + 1
    while code[end].isspace(): end += 1
    assert code[end] == ';'
    scrutinee = prefix[fetch.end():a].strip()
    fetch_body = (f'vm_block = {scrutinee};\n'
                  f'if vm_block == OP_nop {{ vm_block = {mapping[dispatch]}; }}')
    prefix = replace(prefix, [
        (declaration.start(), declaration.end(), f'let mut vm_block: u16 = {mapping[int(declaration[1])]};'),
        (condition.start(1), condition.end(1), mapping[dispatch]),
        (fetch.start(), end + 1, fetch_body),
    ])
    prefix += '\n// Direct C opcode cases; non-opcode control labels are outside the byte range.\n'
    return prefix, nodes, labels, nodes[invalid], {'opcode_regions':len(groups), 'auxiliary_regions':len(auxiliary)}


def replace(source, changes):
    for a, b, value in sorted(changes, reverse=True): source = source[:a] + value + source[b:]
    return source


GOTO = re.compile(r"\bvm_block\s*=\s*(\d+)\s*;\s*continue\s+'vm_dispatch\s*;")


def gotos(body):
    return list(GOTO.finditer(mask(body)))


def local_loop(node, body, roots):
    if node in roots: return body
    edges = [m for m in gotos(body) if int(m[1]) == node]
    if not edges: return body
    body = replace(body, [(m.start(), m.end(), f"continue 'cfg_{node};") for m in edges])
    if f"'cfg_{node}:" not in body:
        body = f"'cfg_{node}: loop {{\n{body}\n}}"
    return body


def variables(body):
    text = mask(body)
    result = set()
    for m in re.finditer(r'\b[A-Za-z_][A-Za-z_0-9]*\b', text):
        left, right = text[:m.start()].rstrip(), text[m.end():].lstrip()
        if left.endswith('.') or left.endswith('::') or right.startswith(':'): continue
        result.add(m[0])
    return result



def addressed_local(place):
    # addr_of!(*pointer.offset(index)) addresses the pointee, not the pointer
    # or index locals. Preserve the owner only for an actual local place (or
    # its field/array projection). Crossing a dereference ends that ownership.
    code = mask(place).strip()
    if code.startswith('*'):
        return set()
    if code.startswith('('):
        end = close(code, 0, '(', ')')
        return addressed_local(code[1:end])
    root = re.match(r'[A-Za-z_][A-Za-z_0-9]*', code)
    if root:
        return {root[0]}
    # Keep unknown syntax conservative rather than moving an escaped owner.
    return variables(place)


def fold(path, localize, direct):
    source = path.read_text()
    code = mask(source)
    entry = int(re.search(r'let mut vm_block: usize = (\d+);', code)[1])
    dispatch = int(re.search(r'while vm_block == (\d+)', code)[1])
    start = code.index('match vm_block {')
    opening = code.index('{', start)
    ending = close(code, opening)
    nodes, leading = {}, {}
    previous = opening + 1
    for m in re.finditer(r'(?m)^(\d+) => \{', code[opening + 1:ending]):
        a = opening + 1 + m.start()
        before = code[opening + 1:a]
        if before.count('{') != before.count('}'): continue
        brace = code.index('{', a)
        end = close(code, brace)
        node = int(m[1]); assert node not in nodes
        gap = source[previous:a]
        assert not mask(gap).strip(' \t\n\r,'), (node, gap)
        leading[node] = gap
        nodes[node] = source[brace + 1:end].strip('\n')
        previous = end + 1
    assert nodes and entry in nodes
    prefix, suffix = source[:opening + 1], source[previous:]
    roots = {entry, dispatch, 0}
    for m in re.finditer(r'\bvm_block\s*=(?!=)\s*', code):
        tail = code[m.end():]
        if re.match(r'\d+\s*;', tail): continue
        assert tail.startswith('match'), tail[:100]
        cursor, end = match_arms(code, m.end() + 5)
        expression = code[cursor:end + 1]
        roots.update(int(n) for n in re.findall(r'=>\s*(\d+)\b', expression))
    if "'vm_dispatch: loop" not in prefix:
        prefix = prefix.replace('\nloop {', "\n'vm_dispatch: loop {", 1)
    for node, body in nodes.items():
        m = mask(body)
        body = replace(body, [(x.start(), x.end(), "continue 'vm_dispatch;")
                              for x in re.finditer(r'\bcontinue\s*;', m)])
        nodes[node] = local_loop(node, body, roots)
    incoming = collections.defaultdict(collections.Counter)
    outgoing = {}
    def add(node, body):
        edges = collections.Counter(int(m[1]) for m in gotos(body))
        outgoing[node] = edges
        for target, count in edges.items(): incoming[target][node] += count
    def remove(node):
        for target, count in outgoing.pop(node).items():
            incoming[target][node] -= count
            if not incoming[target][node]: del incoming[target][node]
    for node, body in nodes.items(): add(node, body)
    count_before = len(nodes)
    queue = list(nodes); heapq.heapify(queue)
    while queue:
        node = heapq.heappop(queue)
        if node not in nodes or node in roots or not incoming[node]: continue
        predecessors = set(incoming[node])
        if node in predecessors: continue
        occurrences = sum(incoming[node].values())
        if occurrences != 1:
            # Duplicating small tails exposes the same direct branches as C.
            # Bound expansion and leave cyclic/shared large regions intact.
            if len(nodes[node]) * (occurrences - 1) > 20000: continue
            if len(nodes[node].splitlines()) > 32: continue
            pending, seen = list(outgoing[node]), set()
            cyclic = False
            while pending:
                target = pending.pop()
                if target == node: cyclic = True; break
                if target in seen or target in roots or target not in nodes: continue
                seen.add(target); pending.extend(outgoing[target])
            if cyclic: continue
        replacements = {}
        insertion = '{\n' + leading[node].lstrip(',') + nodes[node] + '\n}'
        affected = set(outgoing[node]) | predecessors
        for predecessor in predecessors:
            edges = [m for m in gotos(nodes[predecessor]) if int(m[1]) == node]
            assert len(edges) == incoming[node][predecessor]
            new_body = replace(nodes[predecessor], [(m.start(), m.end(), insertion) for m in edges])
            replacements[predecessor] = local_loop(predecessor, new_body, roots)
            affected.update(outgoing[predecessor])
        remove(node)
        for predecessor in predecessors: remove(predecessor)
        del nodes[node]
        for predecessor, body in replacements.items():
            nodes[predecessor] = body
            add(predecessor, body)
        for n in affected: heapq.heappush(queue, n)
    moved = []
    if localize:
        declarations = list(re.finditer(r'(?m)^let mut (\w+): (.+) = (.+);\n', prefix))
        external = prefix
        for d in reversed(declarations): external = external[:d.start()] + external[d.end():]
        external_names = variables(external + suffix)
        addressed = set()
        for body in nodes.values():
            masked = mask(body)
            for m in re.finditer(r'addr_of(?:_mut)?!\s*\(', masked):
                end = close(masked, m.end() - 1, '(', ')')
                addressed.update(addressed_local(body[m.end():end]))
            addressed.update(re.findall(r'&\s*(?:raw\s+)?(?:mut\s+)?(\w+)', masked))
        uses = collections.defaultdict(set)
        for node, body in nodes.items():
            for name in variables(body): uses[name].add(node)
        removals = []
        for d in declarations:
            name, typ = d[1], d[2]
            if name in addressed or name in external_names or '[' in typ or len(uses[name]) != 1: continue
            node = next(iter(uses[name]))
            nodes[node] = f'let mut {name}: {typ};\n' + nodes[node]
            removals.append((d.start(), d.end(), ''))
            moved.append({'name':name, 'node':node, 'declaration':d[0]})
        prefix = replace(prefix, removals)
    assert set(int(m[1]) for body in nodes.values() for m in gotos(body)) <= set(nodes) | {dispatch}
    labels, invalid_body, direct_stats = {n:str(n) for n in nodes}, None, {}
    if direct:
        prefix, nodes, labels, invalid_body, direct_stats = direct_dispatch(prefix, nodes)
    result = prefix + '\n' + ''.join(leading[n].lstrip(',') + f'{labels[n]} => {{\n{nodes[n]}\n}}\n' for n in sorted(nodes))
    if invalid_body is not None:
        # Unknown byte opcodes take the same C invalid-opcode handler; only
        # impossible internal control labels retain the original abort fallback.
        result += '\n0..=255 => {\n' + invalid_body + '\n}\n'
    result += suffix
    path.write_text(result)
    return {'path':str(path), 'blocks_before':count_before, 'blocks_after':len(nodes),
            'roots':len(roots), 'localized':moved, **direct_stats}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('paths', nargs='+', type=pathlib.Path)
    parser.add_argument('--localize', action='store_true')
    parser.add_argument('--direct-opcodes', action='store_true')
    args = parser.parse_args()
    print(json.dumps([fold(p, args.localize, args.direct_opcodes) for p in args.paths], indent=2))
