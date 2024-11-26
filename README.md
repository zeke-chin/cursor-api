
## 项目简介

- 本项目提供了一个代理服务，可以将 Cursor 编辑器的 AI 能力转换为与 OpenAI API 兼容的接口，让您能够在其他应用中复用 Cursor 的 AI 能力。
- 目前只完成rs-capi的开发，go 未实现
- 支持图片


## 使用前准备

1. 访问 [www.cursor.com](https://www.cursor.com) 并完成注册登录（赠送500次快速响应，可通过删除账号再注册重置）
2. 在浏览器中打开开发者工具（F12）
3. 找到 应用-Cookies 中名为 `WorkosCursorSessionToken` 的值并保存(相当于openai的密钥)

## 接口说明

### 基础配置

- 接口地址：`http://localhost:3000/v1/chat/completions`
- 请求方法：POST
- 认证方式：Bearer Token（使用 WorkosCursorSessionToken 的值，支持英文逗号分隔的key入参）
- 请求格式和响应格式参考openai 支持图片！！

## 快速开始
```
docker run --rm -p 3000:3000 ghcr.io/zeke-chin/cursor-api
```

docker-compose
```
services:
  rs-capi:
    image: ghcr.io/zeke-chin/cursor-api:latest
    ports:
      - 7000:3000
```

调用示例
```
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer user_xxxx" \
  -d '{
    "model": "gpt-4o",
    "messages": [
      {
        "role": "user",
        "content": [
          {
            "type": "text",
            "text": "What'\''s in this image? 中文回复"
          },
          {
            "type": "image_url",
            "image_url": {
              "url": "https://zh.wikipedia.org/zh-cn/%E7%BE%8E%E5%85%83#/media/File:50_USD_Series_2004_Note_Back.jpg"
            }
          }
        ]
      }
    ],
    "max_tokens": 300
  }'  | jq
------------------
{
  "id": "chatcmpl-30dc37e7-d411-4946-a24d-dcc78ec2fdec",
  "object": "chat.completion",
  "created": 1732607371,
  "model": "gpt-4o",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "这是一张50美元纸币的背面图像。纸币上印有美国国会大厦的图案。"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 0,
    "completion_tokens": 0,
    "total_tokens": 0
  }
}
```

## 注意事项

- 请妥善保管您的 WorkosCursorSessionToken，不要泄露给他人
- 本项目仅供学习研究使用，请遵守 Cursor 的使用条款

## 原始项目

- 本项目基于 [cursorToApi](https://github.com/luolazyandlazy/cursorToApi) 项目进行优化，感谢原作者的贡献

## 许可证

MIT License
