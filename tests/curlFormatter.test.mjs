import assert from 'node:assert/strict'
import { after, test } from 'node:test'
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'
import { pathToFileURL } from 'node:url'
import ts from 'typescript'

const fixtureDir = mkdtempSync(join(tmpdir(), 'dev-tools-curl-'))
after(() => rmSync(fixtureDir, { recursive: true, force: true }))
const source = readFileSync(new URL('../src/utils/curlFormatter.ts', import.meta.url), 'utf8')
const compiled = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.ESNext } }).outputText
writeFileSync(join(fixtureDir, 'curlFormatter.mjs'), compiled)
const { formatCurl } = await import(pathToFileURL(join(fixtureDir, 'curlFormatter.mjs')).href)

// A fake curl captures argv. No generated command can send an HTTP request.
writeFileSync(join(fixtureDir, 'curl'), '#!/usr/bin/env node\nprocess.stdout.write(JSON.stringify(process.argv.slice(2)))\n', { mode: 0o755 })

function shellArguments(command) {
  const run = spawnSync('/bin/sh', ['-c', command], {
    encoding: 'utf8', env: { ...process.env, PATH: `${fixtureDir}:${process.env.PATH}` }, timeout: 5000,
  })
  assert.equal(run.status, 0, run.stderr)
  return JSON.parse(run.stdout)
}
function argumentValue(args, flag) {
  assert.ok(args.includes(flag), `Missing expected flag: ${flag}`)
  return args[args.indexOf(flag) + 1]
}
const shellTest = process.platform === 'win32' ? test.skip : test

shellTest('modern JSON options preserve SQL apostrophes, template placeholders, Unicode and large IDs byte for byte', () => {
  const payload = String.raw`{"sql":"SELECT * WHERE status = 'ready' AND n > ${'${pageSize}'}","id":9007199254740993,"text":"中文 $HOME $(touch fake) ` + '`date`' + String.raw`","escaped":"a\nb"}`
  const quoted = `'${payload.replaceAll("'", "'\\''")}'`
  for (const option of ['--json', '--data-raw']) {
    for (const bodyOption of ['-d', '--data-binary']) {
      for (const multiline of [true, false]) {
        const result = formatCurl(`curl 'https://example.com/api?x=1&y=2' ${option} ${quoted}`, { bodyOption, multiline })
        const args = shellArguments(result.command)
        assert.equal(argumentValue(args, bodyOption), payload)
        assert.equal(argumentValue(args, '--url'), 'https://example.com/api?x=1&y=2')
        assert.ok(!args.includes('--json') && !args.includes('--data-raw'))
        assert.equal(result.bodyBytes, Buffer.byteLength(payload))
        assert.equal(result.method, 'POST')
        assert.ok(args.includes('Content-Type: application/json'))
        assert.equal(args.includes('Accept: application/json'), option === '--json')
      }
    }
  }
})

shellTest('repairs the supplied paste pattern: Markdown URL, SQL backslash apostrophes, trailing Chinese instructions', () => {
  const input = String.raw`curl --url '[https://example.com/api](https://example.com/api)' \
    -H 'Content-Type: application/json' \
    --data-raw '{"sql":"WHERE code = \'26\' AND t >= \'${'${start}'}\'","id":9007199254740993}' \
    --insecure 换成 Linux curl 可以执行`
  const result = formatCurl(input)
  const args = shellArguments(result.command)
  assert.equal(argumentValue(args, '--url'), 'https://example.com/api')
  assert.equal(argumentValue(args, '--data-binary'), String.raw`{"sql":"WHERE code = '26' AND t >= '${'${start}'}'","id":9007199254740993}`)
  assert.ok(args.includes('--insecure'))
  assert.equal(result.warnings.length, 0)
  assert.ok(result.changes.some(change => change.includes('Markdown')))
  assert.ok(result.changes.some(change => change.includes('转义')))
})

shellTest('understands double-quoted exports, attached values, flags and empty strings', () => {
  const result = formatCurl(String.raw`curl -sSL -XPOST --url="https://example.com/api" -H"X-Name: sample" -H 'X-Empty:' --data-raw "{\"a\":\"line\\nnext\"}"`)
  const args = shellArguments(result.command)
  assert.equal(argumentValue(args, '-X'), 'POST')
  assert.equal(argumentValue(args, '--data-binary'), String.raw`{"a":"line\nnext"}`)
  assert.ok(['-s', '-S', '-L'].every(flag => args.includes(flag)))
  assert.ok(args.includes('X-Name: sample'))
  assert.ok(args.includes('X-Empty:'))
  assert.equal(argumentValue(shellArguments(formatCurl("curl https://example.com -d ''").command), '--data-binary'), '')
})

shellTest('decodes Chrome Bash ANSI-C exports without losing JSON escapes or SQL quotes', () => {
  const payload = String.raw`{"sql":"WHERE name = 'ready'\nAND id = 9007199254740993","path":"C:\\temp","text":"中文"}`
  const ansi = payload.replaceAll('\\', '\\\\').replaceAll("'", "\\'")
  const result = formatCurl(`curl https://example.com --data-raw $'${ansi}'`)
  assert.equal(argumentValue(shellArguments(result.command), '--data-binary'), payload)
  assert.equal(result.warnings.length, 0)
  assert.ok(result.changes.some(change => change.includes('Bash')))
  const escapes = formatCurl(String.raw`curl https://example.com -d $'\x41\u4e2d\t\n'`)
  assert.equal(argumentValue(shellArguments(escapes.command), '--data-binary'), 'A中\t\n')
})

shellTest('existing JSON headers and explicitly suppressed Accept headers are kept', () => {
  const result = formatCurl(`curl https://example.com --json '{}' -H 'content-type: application/vnd.test+json' -H 'Accept:'`)
  const args = shellArguments(result.command)
  assert.equal(result.headerCount, 2)
  assert.ok(!args.includes('Content-Type: application/json'))
  assert.ok(!args.includes('Accept: application/json'))
})

shellTest('multiple --json values concatenate without adding an ampersand', () => {
  const result = formatCurl(`curl https://example.com --json '{"a":' --json '1}'`)
  const args = shellArguments(result.command)
  assert.equal(argumentValue(args, '--data-binary'), '{"a":1}')
  assert.equal(args.filter(arg => arg === '--data-binary').length, 1)
})

shellTest('file body options retain original newline semantics when output mode changes', () => {
  for (const [inputFlag, outputFlag] of [['--data-binary', '--data-binary'], ['-d', '-d'], ['--json', '--data-binary']]) {
    const result = formatCurl(`curl https://example.com ${inputFlag} @payload.json`, { bodyOption: '-d' })
    assert.equal(argumentValue(shellArguments(result.command), outputFlag), '@payload.json')
    assert.ok(result.warnings.length)
  }
})

shellTest('raw text, literal newlines and repeated data options keep their argument values', () => {
  const result = formatCurl("curl https://example.com -d 'first\nsecond' -d 'x=y&z=2'", { bodyOption: '-d' })
  const args = shellArguments(result.command)
  assert.equal(argumentValue(args, '-d'), 'first\nsecond')
  assert.equal(args[args.lastIndexOf('-d') + 1], 'x=y&z=2')
  assert.equal(result.headerCount, 0)
})

test('JSON header inference can be disabled and invalid JSON is reported without mutation', () => {
  assert.equal(formatCurl(`curl https://example.com --data-raw '{}'`, { addJsonContentType: false }).headerCount, 0)
  const result = formatCurl(`curl https://example.com --json '{bad}'`)
  assert.ok(result.warnings.some(warning => warning.includes('有效 JSON')))
  assert.ok(result.command.includes("'{bad}'"))
})

test('GET, HEAD, explicit methods, forms and file upload infer the request method', () => {
  for (const [suffix, method] of [['', 'GET'], ['-I', 'HEAD'], ["-X DELETE", 'DELETE'], ["-G -d 'a=b'", 'GET'], ["-F 'a=b'", 'POST'], ['-T file.txt', 'PUT']]) {
    assert.equal(formatCurl(`curl https://example.com ${suffix}`).method, method)
  }
})

test('supports Markdown fences, terminal prompt and CRLF continuations', () => {
  const result = formatCurl("```bash\n$ curl https://example.com \\\r\n --data-raw '{}'\n```")
  assert.equal(result.url, 'https://example.com')
  assert.equal(result.method, 'POST')
})

test('newer or unknown flags cannot silently pass as 7.29 compatible', () => {
  for (const option of ['--http2', '--fail-with-body', '--tlsv1.2', '--made-up']) {
    assert.throws(() => formatCurl(`curl https://example.com ${option}`), /7.29/)
  }
  const result = formatCurl('curl https://example.com --http1.1')
  assert.ok(!result.command.includes('--http1.1'))
  assert.ok(result.changes.some(change => change.includes('HTTP/1.1')))
})

test('rejects ambiguous raw @ bodies, mixed JSON modes and header files', () => {
  for (const input of [
    "curl https://example.com --data-raw '@literal'",
    "curl https://example.com --json '{}' -d 'x=y'",
    "curl https://example.com --json @a.json --json @b.json",
    "curl https://example.com -H @headers.txt",
  ]) assert.throws(() => formatCurl(input))
})

test('rejects malformed input, shell programs, substitutions and multiple requests', () => {
  for (const input of [
    '', 'wget https://example.com', 'curl -H', 'curl https://example.com --url https://other.example.com',
    'curl https://example.com https://other.example.com', "curl https://example.com -d 'unclosed",
    'curl https://example.com https://other.example.com/中文',
    'curl https://example.com; touch /tmp/should-not-exist', 'curl https://example.com | sh',
    'curl https://example.com?x=1&y=2', 'curl https://example.com -H "Authorization: $TOKEN"',
    'curl https://example.com -d "$(echo test)"', 'curl https://example.com `date`',
    'curl https://example.com -H "X: $?"', String.raw`curl https://example.com -d $'\xFF'`,
    String.raw`curl https://example.com -d $'\0'`,
  ]) assert.throws(() => formatCurl(input), undefined, input)
})
