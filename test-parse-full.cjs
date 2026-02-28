// Test the full parseMarkdown pipeline
const MarkdownIt = require('markdown-it');

// Simulate the parser logic
const md = new MarkdownIt('default', { html: false, breaks: true });

const tableMarkdown = `
| Feature | Status |
| --- | --- |
| Links | ✅ |
| Tables | ✅ |
`;

const tokens = md.parse(tableMarkdown, {});

// Token mappings
const defaultTokens = {
  table_open: { block: 'table' },
  thead_open: { ignore: true, noClose: true },
  thead_close: { ignore: true },
  tbody_open: { ignore: true, noClose: true },
  tbody_close: { ignore: true },
  tr_open: { block: 'tableRow' },
  th_open: { block: 'tableHeader' },
  td_open: { block: 'tableCell' },
  paragraph_open: { block: 'paragraph' },
};

const doc = { type: 'doc', content: [] };
const stack = [doc];

function top() {
  return stack[stack.length - 1];
}

function openBlock(type, attrs) {
  const block = { type, content: [] };
  if (attrs && Object.keys(attrs).length > 0) block.attrs = attrs;
  stack.push(block);
}

function closeBlock() {
  const block = stack.pop();
  if (block && stack.length > 0) {
    const result = { type: block.type };
    if (block.attrs && Object.keys(block.attrs).length > 0) result.attrs = block.attrs;
    if (block.content && block.content.length > 0) result.content = block.content;
    top().content.push(result);
  }
}

function addText(text) {
  if (!text) return;
  top().content.push({ type: 'text', text });
}

// Process tokens
for (let i = 0; i < tokens.length; i++) {
  const token = tokens[i];
  const spec = defaultTokens[token.type];

  console.log(`Processing: ${token.type} (nesting: ${token.nesting})`);

  if (token.nesting === -1) {
    if (token.type.endsWith('_close')) {
      const openTokenType = token.type.replace('_close', '_open');
      const openSpec = defaultTokens[openTokenType];
      if (!openSpec?.noClose) {
        console.log(`  -> Closing block`);
        closeBlock();
      } else {
        console.log(`  -> Ignoring close (noClose flag)`);
      }
    }
    continue;
  }

  if (!spec) {
    if (token.type === 'inline' && token.children) {
      for (const child of token.children) {
        if (child.type === 'text') {
          console.log(`  -> Adding text: "${child.content}"`);
          addText(child.content);
        }
      }
    }
    continue;
  }

  if (spec.ignore) {
    console.log(`  -> Ignored`);
    continue;
  }

  if (spec.block && token.nesting === 1) {
    console.log(`  -> Opening block: ${spec.block}`);
    openBlock(spec.block);
  }
}

// Close remaining blocks
while (stack.length > 1) {
  console.log(`Closing remaining block: ${top().type}`);
  closeBlock();
}

console.log('\n\nFinal document:');
console.log(JSON.stringify(doc, null, 2));
