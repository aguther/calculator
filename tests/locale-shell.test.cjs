const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');

function shell(languages) {
  const nodes = Object.fromEntries(['loading', 'calculator-canvas', 'update'].map(id => [id, {
    attributes: {}, setAttribute(name, value) { this.attributes[name] = value; }
  }]));
  const root = {style: {setProperty() {}}, setAttribute() {}};
  let languageChanged;
  const document = {
    documentElement: root, getElementById: id => nodes[id],
    querySelector: selector => nodes[selector.slice(1)], addEventListener() {}
  };
  vm.runInNewContext(fs.readFileSync('web/app.js', 'utf8'), {
    document, navigator: {languages}, window: {innerHeight: 760, addEventListener() {}},
    MutationObserver: class {
      constructor(callback) { languageChanged = callback; }
      observe() {}
    }
  });
  return {document, nodes, select(language) { root.lang = language; languageChanged(); }};
}

test('shell selects a supported browser language and falls back to English', () => {
  for (const [locales, expected] of [[['de-DE'], 'de'], [['en-GB'], 'en'], [['fr-FR', 'de-AT'], 'de'], [['fr-FR'], 'en']]) {
    const page = shell(locales);
    assert.equal(page.document.documentElement.lang, expected);
    assert.equal(page.nodes.loading.textContent, expected === 'de' ? 'Calculator wird geladen …' : 'Loading calculator…');
  }
});

test('a saved or manually selected language updates shell accessibility and update labels', () => {
  const page = shell(['de-DE']);
  page.select('en');
  assert.equal(page.nodes.update.attributes['aria-label'], 'New version available · update now');
  assert.equal(page.nodes['calculator-canvas'].attributes['aria-label'], 'Time and number calculator. Use digits, operators and Enter.');
  page.select('de');
  assert.equal(page.nodes.update.attributes['aria-label'], 'Neue Version verfügbar · jetzt aktualisieren');
});
