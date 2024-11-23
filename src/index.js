const express = require('express')
const { v4: uuidv4 } = require('uuid')
const { stringToHex, chunkToUtf8String } = require('./utils.js')
require('dotenv').config()
const app = express()

// 中间件配置
app.use(express.json())
app.use(express.urlencoded({ extended: true }))

app.post('/v1/chat/completions', async (req, res) => {
  // o1开头的模型，不支持流式输出
  if (req.body.model.startsWith('o1-') && req.body.stream) {
    return res.status(400).json({
      error: 'Model not supported stream'
    })
  }

  let currentKeyIndex = 0
  try {
    const { model, messages, stream = false } = req.body
    let authToken = req.headers.authorization?.replace('Bearer ', '')
    // 处理逗号分隔的密钥
    const keys = authToken.split(',').map(key => key.trim())
    if (keys.length > 0) {
      // 确保 currentKeyIndex 不会越界
      if (currentKeyIndex >= keys.length) {
        currentKeyIndex = 0
      }
      // 使用当前索引获取密钥
      authToken = keys[currentKeyIndex]
      // 更新索引
      currentKeyIndex = (currentKeyIndex + 1)
    }
    if (authToken && authToken.includes('%3A%3A')) {
      authToken = authToken.split('%3A%3A')[1]
    }
    if (!messages || !Array.isArray(messages) || messages.length === 0 || !authToken) {
      return res.status(400).json({
        error: 'Invalid request. Messages should be a non-empty array and authorization is required'
      })
    }

    const formattedMessages = messages.map(msg => `${msg.role}:${msg.content}`).join('\n')
    const hexData = stringToHex(formattedMessages, model)

    const response = await fetch('https://api2.cursor.sh/aiserver.v1.AiService/StreamChat', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/connect+proto',
        authorization: `Bearer ${authToken}`,
        'connect-accept-encoding': 'gzip,br',
        'connect-protocol-version': '1',
        'user-agent': 'connect-es/1.4.0',
        'x-amzn-trace-id': `Root=${uuidv4()}`,
        'x-cursor-checksum': 'zo6Qjequ9b9734d1f13c3438ba25ea31ac93d9287248b9d30434934e9fcbfa6b3b22029e/7e4af391f67188693b722eff0090e8e6608bca8fa320ef20a0ccb5d7d62dfdef',
        'x-cursor-client-version': '0.42.3',
        'x-cursor-timezone': 'Asia/Shanghai',
        'x-ghost-mode': 'false',
        'x-request-id': uuidv4(),
        Host: 'api2.cursor.sh'
      },
      body: hexData
    })

    if (stream) {
      res.setHeader('Content-Type', 'text/event-stream')
      res.setHeader('Cache-Control', 'no-cache')
      res.setHeader('Connection', 'keep-alive')

      const responseId = `chatcmpl-${uuidv4()}`

      // 使用封装的函数处理 chunk
      for await (const chunk of response.body) {
        const text = chunkToUtf8String(chunk)

        if (text.length > 0) {
          res.write(`data: ${JSON.stringify({
                        id: responseId,
                        object: 'chat.completion.chunk',
                        created: Math.floor(Date.now() / 1000),
                        model,
                        choices: [{
                            index: 0,
                            delta: {
                                content: text
                            }
                        }]
                    })}\n\n`)
        }
      }

      res.write('data: [DONE]\n\n')
      return res.end()
    } else {
      let text = ''
      // 在非流模式下也使用封装的函数
      for await (const chunk of response.body) {
        text += chunkToUtf8String(chunk)
      }
      // 对解析后的字符串进行进一步处理
      text = text.replace(/^.*<\|END_USER\|>/s, '')
      text = text.replace(/^\n[a-zA-Z]?/, '').trim()
      console.log(text)

      return res.json({
        id: `chatcmpl-${uuidv4()}`,
        object: 'chat.completion',
        created: Math.floor(Date.now() / 1000),
        model,
        choices: [{
          index: 0,
          message: {
            role: 'assistant',
            content: text
          },
          finish_reason: 'stop'
        }],
        usage: {
          prompt_tokens: 0,
          completion_tokens: 0,
          total_tokens: 0
        }
      })
    }
  } catch (error) {
    console.error('Error:', error)
    if (!res.headersSent) {
      if (req.body.stream) {
        res.write(`data: ${JSON.stringify({ error: 'Internal server error' })}\n\n`)
        return res.end()
      } else {
        return res.status(500).json({ error: 'Internal server error' })
      }
    }
  }
})

// 启动服务器
const PORT = process.env.PORT || 3000
app.listen(PORT, () => {
  console.log(`服务器运行在端口 ${PORT}`)
})
