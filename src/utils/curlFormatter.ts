export type CurlBodyOption = '-d' | '--data-binary'

export interface CurlFormatOptions {
  bodyOption: CurlBodyOption
  multiline: boolean
  addJsonContentType: boolean
}

export interface CurlFormatResult {
  command: string
  method: string
  url: string
  headerCount: number
  bodyBytes: number
  changes: string[]
  warnings: string[]
}

interface CurlArgument {
  option: string
  value?: string
}

const BODY_OPTIONS = new Set(['--data-raw', '--json', '--data', '-d', '--data-binary', '--data-ascii'])
// Deliberately limited to options available in curl 7.29. Newer options are
// converted explicitly below or rejected, rather than silently discarded.
const VALUE_OPTIONS = new Set([
  '--url', '--request', '-X', '--header', '-H', '--cookie', '-b', '--cookie-jar', '-c',
  '--user', '-u', '--user-agent', '-A', '--referer', '-e', '--output', '-o',
  '--connect-timeout', '--max-time', '-m', '--retry', '--retry-delay', '--retry-max-time',
  '--proxy', '-x', '--proxy-user', '-U', '--noproxy', '--cacert', '--capath',
  '--cert', '-E', '--cert-type', '--key', '--key-type', '--pass', '--ciphers',
  '--interface', '--resolve', '--form', '-F', '--form-string', '--upload-file', '-T',
  '--data-urlencode', '--range', '-r', '--write-out', '-w', '--dump-header', '-D',
  '--limit-rate', '--max-redirs', '--max-filesize', '--stderr', '--netrc-file',
  ...BODY_OPTIONS,
])
const FLAG_OPTIONS = new Set([
  '--location', '-L', '--insecure', '-k', '--silent', '-s', '--show-error', '-S',
  '--include', '-i', '--head', '-I', '--get', '-G', '--compressed', '--globoff', '-g',
  '--fail', '-f', '--verbose', '-v', '--ipv4', '-4', '--ipv6', '-6', '--http1.0', '-0',
  '--tlsv1', '-1', '--basic', '--digest', '--ntlm', '--anyauth', '--location-trusted',
  '--no-buffer', '-N', '--no-keepalive', '--tcp-nodelay', '--raw', '--remote-name', '-O',
  '--remote-header-name', '-J', '--netrc', '-n', '--netrc-optional', '--disable', '-q',
  '--http1.1',
])

function addUnique(items: string[], message: string): void {
  if (!items.includes(message)) items.push(message)
}

function isBodyToken(value: string, previous?: string): boolean {
  return BODY_OPTIONS.has(previous ?? '') || [...BODY_OPTIONS].some(option => value.startsWith(`${option}=`))
    || (value.startsWith('-d') && !value.startsWith('--'))
}

/** Read shell words as data. No shell, eval, request or variable expansion. */
function tokenizeCurl(source: string, changes: string[]): string[] {
  const tokens: string[] = []
  let value = ''
  let quote: "'" | '"' | 'ansi' | null = null
  let started = false

  function flush(): void {
    if (started) tokens.push(value)
    value = ''
    started = false
  }

  for (let index = 0; index < source.length; index++) {
    const char = source[index]
    const next = source[index + 1]
    if (char === '\0') throw new Error('命令包含 NUL 字符，无法表示为 Shell 参数。')

    if (quote === 'ansi') {
      if (char === "'") quote = null
      else if (char !== '\\') value += char
      else {
        if (next === undefined) throw new Error('Bash 引号中的转义未完成。')
        const escapes: Record<string, string> = {
          a: '\x07', b: '\b', e: '\x1b', E: '\x1b', f: '\f', n: '\n',
          r: '\r', t: '\t', v: '\v', '\\': '\\', "'": "'", '"': '"', '?': '?',
        }
        if (Object.prototype.hasOwnProperty.call(escapes, next)) { value += escapes[next]; index++ }
        else if (/[0-7xuU]/.test(next)) {
          const octal = /[0-7]/.test(next)
          const pattern = octal ? /^[0-7]{1,3}/ : next === 'x' ? /^[\da-fA-F]{1,2}/
            : next === 'u' ? /^[\da-fA-F]{1,4}/ : /^[\da-fA-F]{1,8}/
          const start = index + (octal ? 1 : 2)
          const digits = pattern.exec(source.slice(start))?.[0]
          if (!digits) { value += `\\${next}`; index++; continue }
          const code = parseInt(digits, octal ? 8 : 16)
          if (code === 0 || code > 0x10ffff || (code >= 0xd800 && code <= 0xdfff)) {
            throw new Error('Bash 转义包含 NUL 或无效 Unicode，无法无损转换。')
          }
          if ((octal || next === 'x') && code >= 128) {
            throw new Error('Bash 十六进制 / 八进制包含非 ASCII 字节，请使用 UTF-8 原文或 Unicode 转义。')
          }
          value += String.fromCodePoint(code)
          index = start + digits.length - 1
        } else if (next === 'c' || next === '\n' || next === '\r') {
          throw new Error('暂不支持该 Bash 控制字符转义，请改为字面正文。')
        } else { value += `\\${next}`; index++ }
      }
      continue
    }

    if (quote === "'") {
      // Some copied commands put SQL \' inside a single-quoted JSON argument.
      // This is invalid POSIX quoting; repair only JSON-looking body words.
      if (char === '\\' && next === "'"
        && isBodyToken(value, tokens[tokens.length - 1])
        && /^\s*[{[]/.test(value.replace(/^--[\w-]+=/, '').replace(/^-d(?=[{[])/, ''))) {
        value += "'"
        index++
        addUnique(changes, '修复 JSON / SQL 中单引号的粘贴转义，并重新生成 Linux Shell 引号。')
      } else if (char === "'") {
        quote = null
      } else {
        value += char
      }
      continue
    }

    if (char === '\\') {
      if (next === '\n') { index++; continue }
      if (next === '\r' && source[index + 2] === '\n') { index += 2; continue }
      if (next === undefined) throw new Error('命令以未完成的反斜杠结尾。')
      if (quote === '"' && !['$', '`', '"', '\\'].includes(next)) {
        value += char
      } else {
        value += next
        started = true
        index++
      }
      continue
    }

    if (quote === null && char === '$' && next === "'") {
      quote = 'ansi'
      started = true
      index++
      addUnique(changes, '转换 Bash 专用引号，保留正文转义含义。')
      continue
    }
    if (char === '`' || (char === '$' && /[\w{('?$!@*#-]/.test(next ?? ''))) {
      throw new Error('包含 Shell 变量或命令替换；请先替换为实际值，或将字面内容放入普通单引号。')
    }

    if (quote === '"') {
      if (char === '"') quote = null
      else value += char
      continue
    }
    if (char === "'" || char === '"') {
      quote = char
      started = true
    } else if (/\s/.test(char)) {
      flush()
    } else if (/[;|&<>]/.test(char)) {
      throw new Error('请只粘贴一条 cURL 命令；URL 中的 & 等 Shell 特殊字符需要用引号包住。')
    } else if (char === '#' && !started) {
      while (index < source.length && source[index] !== '\n') index++
    } else {
      value += char
      started = true
    }
  }
  if (quote) throw new Error('命令引号未闭合，请检查请求头、JSON 和 SQL 中的单引号。')
  flush()
  return tokens
}

function quoteShell(value: string): string {
  return `'${value.replace(/'/g, "'\\''")}'`
}

function normalizeUrl(value: string, changes: string[]): string {
  const markdown = /^\[(https?:\/\/[^\]]+)\]\((https?:\/\/[^)]+)\)$/.exec(value)
  const url = markdown ? markdown[2] : value
  if (markdown) addUnique(changes, '已将 Markdown 链接恢复为纯 URL。')
  if (!/^https?:\/\/[^\s]+$/i.test(url)) throw new Error('URL 必须是完整的 http:// 或 https:// 地址。')
  return url
}

export function formatCurl(source: string, options: Partial<CurlFormatOptions> = {}): CurlFormatResult {
  const { bodyOption = '--data-binary', multiline = true, addJsonContentType = true } = options
  const changes: string[] = []
  const warnings: string[] = []
  let input = source.trim()
  if (!input) throw new Error('请先粘贴需要转换的 cURL 命令。')
  const fence = /^```(?:bash|sh|shell|curl)?\s*\n([\s\S]*?)\n```$/i.exec(input)
  if (fence) input = fence[1]
  input = input.replace(/^\$\s+(?=curl\b)/, '')
  const tokens = tokenizeCurl(input, changes)
  if (!/^(?:.*\/)?curl(?:\.exe)?$/i.test(tokens[0] ?? '')) {
    throw new Error('请输入以 curl 开头的单条命令。')
  }
  const args: CurlArgument[] = []
  let url = ''
  for (let index = 1; index < tokens.length; index++) {
    const token = tokens[index]
    if (!token.startsWith('-')) {
      if (url && /^[\u3400-\u9fff]/.test(token)) {
        addUnique(changes, '已移除命令末尾的中文操作说明。')
        break
      }
      if (url) throw new Error('只支持一条命令和一个 URL，请移除多余的地址或说明文字。')
      url = normalizeUrl(token, changes)
      args.push({ option: '--url', value: url })
      continue
    }

    let option = token
    let attached: string | undefined
    if (token.startsWith('--') && token.includes('=')) {
      const equals = token.indexOf('=')
      option = token.slice(0, equals)
      attached = token.slice(equals + 1)
    } else if (!token.startsWith('--') && token.length > 2) {
      const short = token.slice(0, 2)
      if (VALUE_OPTIONS.has(short)) {
        option = short
        attached = token.slice(2)
      } else if ([...token.slice(1)].every(char => FLAG_OPTIONS.has(`-${char}`))) {
        args.push(...[...token.slice(1)].map(char => ({ option: `-${char}` })))
        continue
      }
    }

    if (VALUE_OPTIONS.has(option)) {
      const value = attached ?? tokens[++index]
      if (value === undefined) throw new Error(`参数 ${option} 缺少值。`)
      if (option === '--url') {
        if (url) throw new Error('只支持一个 URL，请移除重复地址。')
        url = normalizeUrl(value, changes)
        args.push({ option, value: url })
      } else {
        args.push({ option, value })
      }
    } else if (FLAG_OPTIONS.has(option) && attached === undefined) {
      if (option === '--http1.1') {
        addUnique(changes, '移除 --http1.1：cURL 7.29 默认使用 HTTP/1.1，无需该较新参数。')
      } else args.push({ option })
    } else {
      throw new Error(`7.29 兼容转换暂不支持参数 ${option}，请检查参数或确认服务器版本；不会静默删除该参数。`)
    }
  }
  if (!url) throw new Error('命令中缺少 HTTP / HTTPS URL。')

  const headers = args.filter(arg => arg.option === '-H' || arg.option === '--header')
  if (headers.some(arg => arg.value?.startsWith('@'))) {
    throw new Error('7.29 不支持从 @文件 读取请求头，请将请求头展开为 -H 参数。')
  }
  const bodies = args.filter(arg => BODY_OPTIONS.has(arg.option))
  const jsonBodies = bodies.filter(arg => arg.option === '--json')
  const otherBody = args.some(arg => ['--data-urlencode', '--form', '-F', '--form-string', '--upload-file', '-T'].includes(arg.option))
  if (jsonBodies.length && (jsonBodies.length !== bodies.length || otherBody)) {
    throw new Error('--json 与其他请求体参数混用，无法无损转换；请先合并为一个请求体。')
  }
  if (jsonBodies.length > 1 && jsonBodies.some(arg => arg.value?.startsWith('@'))) {
    throw new Error('多个 --json 含文件引用，无法在离线状态合并；请先合并为一个 JSON 或一个文件。')
  }
  if (bodies.some(arg => arg.option === '--data-raw' && arg.value?.startsWith('@'))) {
    throw new Error('--data-raw 的正文以 @ 开头，直接替换会变成读取文件；请先将正文保存为文件并使用 --data-binary @文件。')
  }

  function hasHeader(name: string): boolean {
    return headers.some(arg => arg.value?.split(/[:;]/, 1)[0].trim().toLowerCase() === name)
  }
  function addHeader(name: string): void {
    if (hasHeader(name.toLowerCase())) return
    const header = { option: '-H', value: `${name}: application/json` }
    args.push(header)
    headers.push(header)
    changes.push(`补充 ${name}: application/json。`)
  }

  // Parsing here is validation only: never reserialize JSON (large IDs and SQL
  // escapes must keep their original bytes).
  const inlineBodies = bodies.filter(arg => !arg.value?.startsWith('@') || arg.option === '--data-raw')
  const bodyText = inlineBodies.map(arg => arg.value ?? '').join(jsonBodies.length ? '' : '&')
  let isJson = false
  if (bodies.length && inlineBodies.length === bodies.length) {
    try { JSON.parse(bodyText); isJson = true } catch { /* Non-JSON data is valid curl input. */ }
  }
  const expectsJson = jsonBodies.length > 0 || headers.some(arg => /^content-type\s*:\s*[^;]*json/i.test(arg.value ?? ''))
  if (bodies.length && expectsJson && inlineBodies.length === bodies.length && !isJson) {
    warnings.push('请求体不是有效 JSON，已保留原始文本，请核对后再执行。')
  }
  if (jsonBodies.length) {
    addHeader('Content-Type')
    addHeader('Accept')
  } else if (addJsonContentType && isJson) addHeader('Content-Type')

  const converted: CurlArgument[] = []
  let emittedJson = false
  for (const arg of args) {
    if (!BODY_OPTIONS.has(arg.option)) { converted.push(arg); continue }
    if (arg.option === '--json' && emittedJson) continue
    if (arg.option === '--json') emittedJson = true
    const value = arg.option === '--json' && jsonBodies.length > 1 ? bodyText : arg.value ?? ''
    const fileReference = value.startsWith('@') && arg.option !== '--data-raw'
    // File reads have different newline handling for -d and --data-binary.
    const outputOption = fileReference ? (arg.option === '--json' ? '--data-binary' : arg.option) : bodyOption
    converted.push({ option: outputOption, value })
    if (arg.option !== outputOption) addUnique(changes, `将 ${arg.option} 转为 ${outputOption}。`)
    if (fileReference) addUnique(warnings, '请求体引用文件或标准输入，保留读取方式；执行时需提供对应文件或输入。')
  }
  if (args.some(arg => ['-F', '--form'].includes(arg.option) && arg.value?.includes(';filename='))) {
    warnings.push('表单文件参数已保留，执行时需提供对应文件。')
  }
  const method = [...args].reverse().find(arg => arg.option === '-X' || arg.option === '--request')?.value
    ?? (args.some(arg => ['-I', '--head'].includes(arg.option)) ? 'HEAD'
      : args.some(arg => ['-G', '--get'].includes(arg.option)) ? 'GET'
        : args.some(arg => ['-T', '--upload-file'].includes(arg.option)) ? 'PUT'
          : bodies.length || otherBody ? 'POST' : 'GET')
  const lines = converted.map(arg => arg.value === undefined ? arg.option : `${arg.option} ${quoteShell(arg.value)}`)
  return {
    command: multiline ? ['curl', ...lines.map(line => `  ${line}`)].join(' \\\n') : ['curl', ...lines].join(' '),
    method, url, headerCount: headers.length,
    bodyBytes: new TextEncoder().encode(bodyText).length,
    changes, warnings,
  }
}
