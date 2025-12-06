/**
 * JSON工具类
 * 提供JSON解析、验证等功能，支持带注释的JSON格式
 */

/**
 * 清理JSON字符串中的注释和多余逗号
 * @param jsonStr 原始JSON字符串
 * @returns 清理后的JSON字符串
 */
export const cleanJsonString = (jsonStr: string): string => {
  return jsonStr
    // 移除单行注释 // 注释内容
    .replace(/\/\/.*$/gm, '')
    // 移除多行注释 /* 注释内容 */
    .replace(/\/\*[\s\S]*?\*\//g, '')
    // 清理多余的空行
    .replace(/\n\s*\n/g, '\n')
    // 移除连续逗号
    .replace(/,(\s*,)/g, ',')
    // 移除尾随逗号（对象和数组结尾）
    .replace(/,(\s*[}\]])/g, '$1')
    // 移除行尾孤立逗号
    .replace(/,(\s*\n\s*[}\]])/g, '$1')
    // 移除属性值后的多余逗号
    .replace(/,(\s*\n\s*})/g, '$1')
    .replace(/,(\s*\n\s*\])/g, '$1')
    // 最终清理：移除所有在 } 或 ] 之前的逗号
    .replace(/,(\s*[}\]])/g, '$1')
    .trim()
}

/**
 * 解析带注释的JSON
 * @param jsonStr JSON字符串
 * @param debug 是否输出调试信息（默认false）
 * @returns 解析后的JSON对象
 */
export const parseJsonWithComments = (jsonStr: string, debug: boolean = false): any => {
  const cleanedJson = cleanJsonString(jsonStr)
  
  // 调试：如果原始JSON和清理后的JSON不同，在控制台输出
  if (debug && jsonStr !== cleanedJson) {
    console.log('JSON清理前后对比:')
    console.log('原始JSON:', jsonStr)
    console.log('清理后JSON:', cleanedJson)
  }
  
  return JSON.parse(cleanedJson)
}

/**
 * 检查字符串是否为有效JSON（支持注释）
 * @param str 待检查的字符串
 * @returns 是否为有效JSON
 */
export const isValidJson = (str: string): boolean => {
  try {
    parseJsonWithComments(str)
    return true
  } catch {
    return false
  }
}

