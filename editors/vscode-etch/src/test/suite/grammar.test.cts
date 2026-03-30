import * as assert from 'node:assert';
import * as fs from 'node:fs/promises';
import * as path from 'node:path';

suite('Etch Grammar', () => {
  test('treats inline math directive content as raw math', async () => {
    const grammar = await loadEtchGrammar();
    const fixture = await loadFixture('inline-math-caret-bleed.etch');
    const nestedFixture = await loadFixture('inline-math-nested-brackets.etch');
    const { repository } = grammar;
    const inlinePatterns = includesOf(repository.inline.patterns);
    const directiveBracketPatterns = includesOf(repository['directive-bracket-content'].patterns);
    const mathBracketPatterns = includesOf(repository['math-directive-bracket-content'].patterns);

    assert.match(
      fixture,
      /:math\[E = mc\^2\] in flowing text\./,
      'Expected the fixture to preserve the inline math caret case that previously bled highlighting.'
    );
    assert.match(
      nestedFixture,
      /:math\[f\(x\[0\]\) \+ g\(\[1, 2, 3\]\)\] without bleeding/,
      'Expected the nested fixture to preserve balanced brackets inside inline math content.'
    );
    assert.ok(repository['math-inline-directive'], 'Expected a dedicated math inline directive rule.');
    assert.ok(
      inlinePatterns.includes('#math-inline-directive'),
      'Expected the top-level inline grammar to include the dedicated math directive rule.'
    );
    assert.ok(
      inlinePatterns.indexOf('#math-inline-directive') <
        inlinePatterns.indexOf('#inline-directive'),
      'Expected the math directive rule to run before the generic inline directive rule.'
    );
    assert.ok(
      directiveBracketPatterns.indexOf('#math-inline-directive') <
        directiveBracketPatterns.indexOf('#inline-directive'),
      'Expected nested directive content to prefer math directives before generic directives.'
    );
    assert.strictEqual(
      repository['math-block-directive'].patterns?.[0]?.include,
      '#math-directive-open-tail',
      'Expected block math directives to use the raw math tail parser.'
    );
    assert.strictEqual(
      repository['math-container-directive'].patterns?.[0]?.include,
      '#math-directive-open-tail',
      'Expected container math directives to use the raw math tail parser.'
    );
    assert.strictEqual(
      repository['math-inline-directive'].patterns?.[0]?.include,
      '#math-directive-bracket-content',
      'Expected inline math directives to use raw math bracket parsing.'
    );
    assert.strictEqual(
      repository['math-directive-bracket-content'].contentName,
      'markup.raw.inline.math.etch',
      'Expected inline math content to get a raw math scope.'
    );
    assert.deepStrictEqual(
      mathBracketPatterns,
      ['#escape', '#math-directive-bracket-content'],
      'Expected inline math content to allow escapes and balanced nested brackets only.'
    );
    assert.strictEqual(
      repository['math-directive-bracket-content'].patterns?.[1]?.include,
      '#math-directive-bracket-content',
      'Expected inline math content to recurse for balanced nested brackets.'
    );
    assert.ok(
      !mathBracketPatterns.includes('#superscript'),
      'Expected inline math content not to include superscript parsing.'
    );
    assert.ok(
      !mathBracketPatterns.includes('#subscript'),
      'Expected inline math content not to include subscript parsing.'
    );
  });
});

type GrammarFile = {
  repository: Repository;
};

type Repository = Record<
  string,
  {
    contentName?: string;
    patterns?: Array<{ include?: string }>;
  }
>;

let grammarPromise: Promise<GrammarFile> | undefined;

async function loadEtchGrammar() {
  grammarPromise ??= createGrammar();
  return grammarPromise;
}

async function createGrammar() {
  const grammarPath = path.resolve(__dirname, '..', '..', '..', 'syntaxes', 'etch.tmLanguage.json');
  return JSON.parse(await fs.readFile(grammarPath, 'utf8')) as GrammarFile;
}

async function loadFixture(name: string): Promise<string> {
  const fixturePath = path.resolve(__dirname, '..', 'fixtures', name);
  return fs.readFile(fixturePath, 'utf8');
}

function includesOf(patterns: Array<{ include?: string }> | undefined): string[] {
  return (patterns ?? []).flatMap((pattern) => (pattern.include ? [pattern.include] : []));
}
