
## 项目简介

本项目提供了一个代理服务，可以将 Cursor 编辑器的 AI 能力转换为与 OpenAI API 兼容的接口，让您能够在其他应用中复用 Cursor 的 AI 能力。

## 使用前准备

1. 访问 [www.cursor.com](https://www.cursor.com) 并完成注册登录（赠送500次快速响应，可通过删除账号再注册重置）
2. 在浏览器中打开开发者工具（F12）
3. 找到 应用-Cookies 中名为 `WorkosCursorSessionToken` 的值并保存(相当于openai的密钥)

## 接口说明

### 基础配置

- 接口地址：`http://localhost:3000/v1/chat/completions`
- 请求方法：POST
- 认证方式：Bearer Token（使用 WorkosCursorSessionToken 的值，支持英文逗号分隔的key入参）
- 请求格式和响应格式参考openai

## 快速开始
```
docker run xxxx -p 3000:3000 ghcr.io/xxxx/rs-capi:latest
```

docker-compose
```
services:
  rs-capi:
    image: ghcr.io/xxxx/rs-capi:latest
    ports:
      - 3000:3000
```

## 注意事项

- 请妥善保管您的 WorkosCursorSessionToken，不要泄露给他人
- 本项目仅供学习研究使用，请遵守 Cursor 的使用条款
- 目前只完成rs-capi的开发，go 未实现

## 原始项目

- 本项目基于 [cursorToApi](https://github.com/luolazyandlazy/cursorToApi) 项目进行优化，感谢原作者的贡献

## 许可证

MIT License