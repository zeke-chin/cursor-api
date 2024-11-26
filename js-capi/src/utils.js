// Helper function to convert string to hex bytes
function stringToHex (str, modelName) {
  const bytes = Buffer.from(str, 'utf-8')
  const byteLength = bytes.length

  // Calculate lengths and fields similar to Python version
  const FIXED_HEADER = 2
  const SEPARATOR = 1
  const FIXED_SUFFIX_LENGTH = 0xA3 + modelName.length

  // 计算文本长度字段 (类似 Python 中的 base_length1)
  let textLengthField1, textLengthFieldSize1
  if (byteLength < 128) {
    textLengthField1 = byteLength.toString(16).padStart(2, '0')
    textLengthFieldSize1 = 1
  } else {
    const lowByte1 = (byteLength & 0x7F) | 0x80
    const highByte1 = (byteLength >> 7) & 0xFF
    textLengthField1 = lowByte1.toString(16).padStart(2, '0') + highByte1.toString(16).padStart(2, '0')
    textLengthFieldSize1 = 2
  }

  // 计算基础长度 (类似 Python 中的 base_length)
  const baseLength = byteLength + 0x2A
  let textLengthField, textLengthFieldSize
  if (baseLength < 128) {
    textLengthField = baseLength.toString(16).padStart(2, '0')
    textLengthFieldSize = 1
  } else {
    const lowByte = (baseLength & 0x7F) | 0x80
    const highByte = (baseLength >> 7) & 0xFF
    textLengthField = lowByte.toString(16).padStart(2, '0') + highByte.toString(16).padStart(2, '0')
    textLengthFieldSize = 2
  }

  // 计算总消息长度
  const messageTotalLength = FIXED_HEADER + textLengthFieldSize + SEPARATOR +
        textLengthFieldSize1 + byteLength + FIXED_SUFFIX_LENGTH

  const messageLengthHex = messageTotalLength.toString(16).padStart(10, '0')

  // 构造完整的十六进制字符串
  const hexString = (
    messageLengthHex +
        '12' +
        textLengthField +
        '0A' +
        textLengthField1 +
        bytes.toString('hex') +
        '10016A2432343163636435662D393162612D343131382D393239612D3936626330313631626432612' +
        '2002A132F643A2F6964656150726F2F656475626F73733A1E0A' +
        // 将模型名称长度转换为两位十六进制，并确保是大写
        Buffer.from(modelName, 'utf-8').length.toString(16).padStart(2, '0').toUpperCase() +
        Buffer.from(modelName, 'utf-8').toString('hex').toUpperCase() +
        '22004A' +
        '24' + '61383761396133342D323164642D343863372D623434662D616636633365636536663765' +
        '680070007A2436393337376535612D386332642D343835342D623564392D653062623232336163303061' +
        '800101B00100C00100E00100E80100'
  ).toUpperCase()
  return Buffer.from(hexString, 'hex')
}

// 封装函数，用于将 chunk 转换为 UTF-8 字符串
function chunkToUtf8String (chunk) {
  if (chunk[0] === 0x01 || chunk[0] === 0x02 || (chunk[0] === 0x60 && chunk[1] === 0x0C)) {
    return ''
  }

  console.log('chunk:', Buffer.from(chunk).toString('hex'))
  console.log('chunk string:', Buffer.from(chunk).toString('utf-8'))

  // 去掉 chunk 中 0x0A 以及之前的字符
  chunk = chunk.slice(chunk.indexOf(0x0A) + 1)

  let filteredChunk = []
  let i = 0
  while (i < chunk.length) {
    // 新的条件过滤：如果遇到连续4个0x00，则移除其之后所有的以 0 开头的字节（0x00 到 0x0F）
    if (chunk.slice(i, i + 4).every(byte => byte === 0x00)) {
      i += 4 // 跳过这4个0x00
      while (i < chunk.length && chunk[i] >= 0x00 && chunk[i] <= 0x0F) {
        i++ // 跳过所有以 0 开头的字节
      }
      continue
    }

    if (chunk[i] === 0x0C) {
      // 遇到 0x0C 时，跳过 0x0C 以及后续的所有连续的 0x0A
      i++ // 跳过 0x0C
      while (i < chunk.length && chunk[i] === 0x0A) {
        i++ // 跳过所有连续的 0x0A
      }
    } else if (
      i > 0 &&
      chunk[i] === 0x0A &&
      chunk[i - 1] >= 0x00 &&
      chunk[i - 1] <= 0x09
    ) {
      // 如果当前字节是 0x0A，且前一个字节在 0x00 至 0x09 之间，跳过前一个字节和当前字节
      filteredChunk.pop() // 移除已添加的前一个字节
      i++ // 跳过当前的 0x0A
    } else {
      filteredChunk.push(chunk[i])
      i++
    }
  }

  // 第二步：去除所有的 0x00 和 0x0C
  filteredChunk = filteredChunk.filter((byte) => byte !== 0x00 && byte !== 0x0C)

  // 去除小于 0x0A 的字节
  filteredChunk = filteredChunk.filter((byte) => byte >= 0x0A)

  const hexString = Buffer.from(filteredChunk).toString('hex')
  console.log('hexString:', hexString)
  const utf8String = Buffer.from(filteredChunk).toString('utf-8')
  console.log('utf8String:', utf8String)
  return utf8String
}

module.exports = {
  stringToHex,
  chunkToUtf8String
}
