export function parseLegacyJson(source) {
  // JSON.parse alone silently accepts repeated keys (last value wins), which
  // could hide a sale reference or replace a commercial state.
  const parsed = JSON.parse(source);
  const stack = [];
  const tokens = /"(?:\\[\s\S]|[^"\\])*"|[{}\[\]]/g;
  for (const match of source.matchAll(tokens)) {
    const token = match[0];
    if (token === '{') stack.push(new Set());
    else if (token === '[') stack.push(null);
    else if (token === '}' || token === ']') stack.pop();
    else {
      const keys = stack.at(-1);
      if (!keys) continue;
      let next = match.index + token.length;
      while (/[ \t\r\n]/.test(source[next] ?? '')) next++;
      if (source[next] !== ':') continue;
      const key = JSON.parse(token);
      if (keys.has(key)) throw Error('Duplicate JSON field; prepare an unambiguous metadata file');
      keys.add(key);
    }
  }
  return parsed;
}
