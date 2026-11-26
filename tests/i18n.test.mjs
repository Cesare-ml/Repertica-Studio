import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { chooseLanguage, translate } from '../src/app/i18n.ts';

const text = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('OS locale maps BCP-47 and unsupported languages fall back to English',()=>{
  assert.equal(chooseLanguage('it-IT'),'it');
  assert.equal(chooseLanguage('de_DE'),'de');
  assert.equal(chooseLanguage('fr-CA'),'fr');
  assert.equal(chooseLanguage('zz-ZZ'),'en');
  assert.equal(chooseLanguage(null),'en');
});
test('missing translations fall back to English, not Italian',()=>{
  assert.equal(translate('Opere','it'),'Opere');
  assert.equal(translate('Opere','en'),'Artworks');
  assert.equal(translate('Opere','es'),'Obras');
  assert.equal(translate('Archivio sul tuo computer','de'),'Archive on this computer');
});
test('all static translated template strings have an English fallback',()=>{
  const source=text('src/app/i18n.ts');
  const block=source.slice(source.indexOf('const EN:'),source.indexOf('const ES:'));
  const keys=new Set([...block.matchAll(/(?:^|,\s*)\s*(?:'((?:\\.|[^'])*)'|"((?:\\.|[^"])*)")\s*:/gm)]
      .map(m=>(m[1]??m[2]).replaceAll("\\'","'")));
  const html=text('src/app/app.component.html');
  const references=[...html.matchAll(/tr\('((?:\\.|[^'])*)'\)/g)].map(m=>m[1].replaceAll("\\'","'"));
  for(const key of references) assert.ok(keys.has(key), 'Missing English UI translation: '+key);
});
test('auction connection screens have an English fallback for every label',()=>{
  const source=text('src/app/i18n.ts');
  const block=source.slice(source.indexOf('const EN:'),source.indexOf('const ES:'));
  const keys=new Set([...block.matchAll(/(?:^|,\s*)\s*(?:'((?:\.|[^'])*)'|"((?:\.|[^"])*)")\s*:/gm)]
      .map(m=>(m[1]??m[2]).replaceAll("\'","'")));
  for(const file of ['src/app/auction-online.component.ts','src/app/auctions.component.ts']){
    for(const m of text(file).matchAll(/tr\('((?:\.|[^'])*)'\)/g)) assert.ok(keys.has(m[1]),'Missing English UI translation: '+m[1]);
  }
});
