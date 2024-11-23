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

// 封装的函数，用于将 chunk 转换为 UTF-8 字符串
function chunkToUtf8String (chunk) {
  if (chunk[0] === 0x01 || chunk[0] === 0x02) {
    return ''
  }
  // 去掉 chunk 中 0x0a 以及之前的字符
  chunk = chunk.slice(chunk.indexOf(0x0a) + 1)
  let hexString = Buffer.from(chunk).toString('hex')
  console.log('hexString:', hexString)

  // 去除里面所有这样的字符：0 跟着一个数字然后 0a，去除掉换页符 0x0c
  hexString = hexString.replace(/0\d0a/g, '').replace(/0c/g, '')
  console.log('hexString2:', hexString)
  const utf8String = Buffer.from(hexString, 'hex').toString('utf-8')
  console.log('utf8String:', utf8String)
  return utf8String
}

module.exports = {
  stringToHex,
  chunkToUtf8String
}
