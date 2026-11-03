import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { spawnSync } from 'node:child_process'
import test from 'node:test'
import ts from 'typescript'

const source = await readFile(new URL('../src/utils/httpTestServer.ts', import.meta.url), 'utf8')
const output = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 } }).outputText
const { defaultTestConfig, testServerMarkdown } = await import('data:text/javascript;base64,' + Buffer.from(output).toString('base64'))

test('Markdown includes configured addresses, arbitrary inputs and fixed responses', () => {
  const config = defaultTestConfig()
  config.port = 9876
  config.endpoints[0].path = '/receive'
  config.endpoints[0].responseBody = '{"received":true}'
  config.endpoints.push({ method: 'HEAD', path: '/status', statusCode: 200, responseBody: '{}' })
  const markdown = testServerMarkdown(config)
  assert.ok(markdown.includes('ANY http://127.0.0.1:9876/receive'))
  assert.ok(markdown.includes('{"received":true}'))
  assert.ok(markdown.includes('无必填字段'))
  assert.ok(markdown.includes("curl --head --url 'http://127.0.0.1:9876/status'"))
  assert.ok(markdown.includes('最近 200 条'))
})

test('curl examples quote URLs without executing shell expressions', () => {
  const config = defaultTestConfig()
  config.endpoints[0].path = "/receive'$(printf)"
  const markdown = testServerMarkdown(config)
  const curl = markdown.split('\n').find(line => line.startsWith('curl '))
  assert.ok(curl)
  const parsed = spawnSync('/bin/sh', ['-c', `set -- ${curl.slice(5)}; printf '%s\\n' "$@"`], { encoding: 'utf8' })
  assert.equal(parsed.status, 0)
  assert.ok(parsed.stdout.split('\n').includes("http://127.0.0.1:8766/receive'$(printf)"))
})

test('JSON containing Markdown fences cannot end the response code block', () => {
  const config = defaultTestConfig()
  config.endpoints[0].responseBody = '{"example":"```"}'
  const markdown = testServerMarkdown(config)
  assert.ok(markdown.includes('````json\n{"example":"```"}\n````'))
})

test('invalid configuration cannot be exported as a working usage document', () => {
  const config = defaultTestConfig()
  config.endpoints[0].responseBody = '{invalid}'
  assert.throws(() => testServerMarkdown(config), /有效 JSON/)
  config.endpoints[0].responseBody = '{}'
  config.endpoints.push({ ...config.endpoints[0], method: 'POST' })
  assert.throws(() => testServerMarkdown(config), /冲突/)
})
