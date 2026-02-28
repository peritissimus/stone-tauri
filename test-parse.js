// Quick test of markdown-it table parsing
const MarkdownIt = require('markdown-it');

const md = new MarkdownIt('default', { html: false, breaks: true });

const tableMarkdown = `
| Feature | Status | Description |
| --- | --- | --- |
| Note Linking | ✅ | Wiki-style \`[[links]]\` |
| Backlinks | ✅ | Bidirectional references |
`;

const tokens = md.parse(tableMarkdown, {});

console.log('Total tokens:', tokens.length);
console.log('\nAll tokens:');
tokens.forEach((token, i) => {
  console.log(`${i}: ${token.type} (tag: ${token.tag}, nesting: ${token.nesting})`);
  if (token.children && token.children.length > 0) {
    console.log(`   Children: ${token.children.length}`);
    token.children.forEach((child, j) => {
      console.log(`     ${j}: ${child.type} "${child.content}"`);
    });
  }
});
